package api

import (
	"bufio"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"os/exec"
	"path/filepath"
	"reflect"
	"sort"
	"strings"
	"sync"
	"time"

	"git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand/workspace"
)

type Job struct {
	ID       string    `json:"id"`
	Action   string    `json:"action"`
	Args     []string  `json:"args"`
	Dir      string    `json:"dir"`
	Status   string    `json:"status"`
	Started  time.Time `json:"started"`
	Finished time.Time `json:"finished,omitempty"`
	ExitCode int       `json:"exit_code,omitempty"`
	Error    string    `json:"error,omitempty"`
	Result   any       `json:"result,omitempty"`
	Lines    []string  `json:"lines,omitempty"`

	resultType  reflect.Type
	mu          sync.Mutex
	subscribers map[chan string]struct{}
}

type jobStore struct {
	mu   sync.RWMutex
	jobs map[string]*Job
}

func newJobStore() *jobStore { return &jobStore{jobs: map[string]*Job{}} }

func newJobID() string {
	var value [16]byte
	if _, err := rand.Read(value[:]); err != nil {
		return fmt.Sprint(time.Now().UnixNano())
	}
	return hex.EncodeToString(value[:])
}

func (s *jobStore) create(action Action, args []string, dir string) *Job {
	job := &Job{ID: newJobID(), Action: action.Name, Args: append([]string(nil), args...), Dir: filepath.ToSlash(dir), Status: "running", Started: time.Now(), subscribers: map[chan string]struct{}{}, resultType: action.Result}
	s.mu.Lock()
	s.jobs[job.ID] = job
	s.mu.Unlock()
	return job
}

func (s *jobStore) get(id string) *Job {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.jobs[id]
}

func (s *jobStore) list() []*Job {
	s.mu.RLock()
	jobs := make([]*Job, 0, len(s.jobs))
	for _, job := range s.jobs {
		jobs = append(jobs, job)
	}
	s.mu.RUnlock()
	sort.Slice(jobs, func(i, j int) bool { return jobs[i].Started.After(jobs[j].Started) })
	return jobs
}

func (s *Server) runJob(job *Job) {
	job.append("$ packwand " + strings.Join(job.Args, " "))
	command := exec.Command(workspace.SelfBin(), job.Args...)
	command.Dir = job.Dir
	workspace.ConfigureSubprocess(command)
	stdout, err := command.StdoutPipe()
	if err != nil {
		job.finish(-1, err)
		return
	}
	stderr, err := command.StderrPipe()
	if err != nil {
		job.finish(-1, err)
		return
	}
	if err = command.Start(); err != nil {
		job.finish(-1, err)
		return
	}

	var stdoutBuffer strings.Builder
	var wait sync.WaitGroup
	wait.Add(2)
	go streamLines(stdout, func(line string) { stdoutBuffer.WriteString(line); stdoutBuffer.WriteByte('\n'); job.append(line) }, &wait)
	go streamLines(stderr, job.append, &wait)
	wait.Wait()
	err = command.Wait()
	exitCode := 0
	if command.ProcessState != nil {
		exitCode = command.ProcessState.ExitCode()
	}
	if err == nil && job.resultType != nil {
		result := reflect.New(job.resultType)
		if decodeErr := json.Unmarshal([]byte(stdoutBuffer.String()), result.Interface()); decodeErr != nil {
			err = fmt.Errorf("decode structured result: %w", decodeErr)
		} else {
			job.mu.Lock()
			job.Result = result.Elem().Interface()
			job.mu.Unlock()
		}
	}
	job.finish(exitCode, err)
}

func streamLines(reader io.Reader, emit func(string), wait *sync.WaitGroup) {
	defer wait.Done()
	scanner := bufio.NewScanner(reader)
	scanner.Buffer(make([]byte, 64*1024), 1024*1024)
	for scanner.Scan() {
		emit(scanner.Text())
	}
	if err := scanner.Err(); err != nil {
		emit("stream error: " + err.Error())
	}
}

func (j *Job) append(line string) {
	j.mu.Lock()
	j.Lines = append(j.Lines, line)
	for subscriber := range j.subscribers {
		select {
		case subscriber <- line:
		default:
		}
	}
	j.mu.Unlock()
}

func (j *Job) finish(exitCode int, err error) {
	j.mu.Lock()
	j.ExitCode, j.Finished = exitCode, time.Now()
	if err != nil {
		j.Status, j.Error = "failed", err.Error()
	} else {
		j.Status = "completed"
	}
	line := "completed"
	if err != nil {
		line = "failed: " + err.Error()
	}
	j.Lines = append(j.Lines, line)
	for subscriber := range j.subscribers {
		select {
		case subscriber <- line:
		default:
		}
		close(subscriber)
	}
	j.subscribers = map[chan string]struct{}{}
	j.mu.Unlock()
}

func (j *Job) subscribe() ([]string, chan string) {
	channel := make(chan string, 128)
	j.mu.Lock()
	replay := append([]string(nil), j.Lines...)
	if j.Status == "running" {
		j.subscribers[channel] = struct{}{}
	} else {
		close(channel)
	}
	j.mu.Unlock()
	return replay, channel
}

func (j *Job) unsubscribe(channel chan string) {
	j.mu.Lock()
	if _, ok := j.subscribers[channel]; ok {
		delete(j.subscribers, channel)
		close(channel)
	}
	j.mu.Unlock()
}
