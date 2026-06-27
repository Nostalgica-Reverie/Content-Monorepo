package main

import (
	"fmt"
	"sort"
	"sync"
)

type Resource string

type Task struct {
	Name     string
	Priority int
	Needs    []Resource
	Run      func() error
}

type Scheduler struct {
	mu      sync.Mutex
	cond    *sync.Cond
	queues  map[Resource][]*sTask
	workers int
	pending int
	seq     uint64
	readyQ  []*sTask
	inReady map[*sTask]bool
	closed  bool
}

type sTask struct {
	Task
	seq   uint64
	doneC chan error
}

func NewScheduler(workers int) *Scheduler {
	if workers < 1 {
		workers = 1
	}
	s := &Scheduler{
		queues:  make(map[Resource][]*sTask),
		workers: workers,
		inReady: make(map[*sTask]bool),
	}
	s.cond = sync.NewCond(&s.mu)
	for i := 0; i < workers; i++ {
		go s.worker()
	}
	return s
}

func (s *Scheduler) worker() {
	s.mu.Lock()
	defer s.mu.Unlock()
	for {
		for len(s.readyQ) == 0 {
			if s.pending == 0 && s.closed {
				s.cond.Broadcast()
				return
			}
			s.cond.Wait()
		}
		st := s.readyQ[0]
		s.readyQ = s.readyQ[1:]
		delete(s.inReady, st)

		if !s.canRunResources(st) {
			continue
		}
		s.mu.Unlock()

		err := safeRun(st.Run)

		s.mu.Lock()
		released := st.Needs
		for _, r := range released {
			if q, ok := s.queues[r]; ok {
				s.queues[r] = removeTask(q, st)
				if len(s.queues[r]) == 0 {
					delete(s.queues, r)
				}
			}
		}
		s.pending--
		s.promoteReady(released)
		s.cond.Broadcast()
		st.doneC <- err
		close(st.doneC)
	}
}

func (s *Scheduler) Submit(t Task) <-chan error {
	needs := uniqueResources(t.Needs)
	st := &sTask{
		Task: Task{
			Name:     t.Name,
			Priority: t.Priority,
			Needs:    needs,
			Run:      t.Run,
		},
		doneC: make(chan error, 1),
	}

	s.mu.Lock()
	s.seq++
	st.seq = s.seq
	s.pending++
	for _, r := range needs {
		s.queues[r] = insertByPriority(s.queues[r], st)
	}
	if s.canRunResources(st) {
		s.readyQ = append(s.readyQ, st)
		s.inReady[st] = true
	}
	s.mu.Unlock()

	s.cond.Broadcast()
	return st.doneC
}

func (s *Scheduler) SubmitWait(t Task) error {
	return <-s.Submit(t)
}

// Close signals that no more tasks will be submitted. Workers exit once all
// pending tasks finish. Must be called after the last Submit.
func (s *Scheduler) Close() {
	s.mu.Lock()
	s.closed = true
	s.cond.Broadcast()
	s.mu.Unlock()
}

func (s *Scheduler) Wait() {
	s.mu.Lock()
	for s.pending > 0 {
		s.cond.Wait()
	}
	s.mu.Unlock()
}

func (s *Scheduler) canRunResources(st *sTask) bool {
	for _, r := range st.Needs {
		q := s.queues[r]
		if len(q) == 0 || q[0] != st {
			return false
		}
	}
	return true
}

func (s *Scheduler) promoteReady(released []Resource) {
	seen := map[*sTask]bool{}
	for _, r := range released {
		q := s.queues[r]
		if len(q) == 0 {
			continue
		}
		head := q[0]
		if seen[head] || s.inReady[head] {
			continue
		}
		seen[head] = true
		if s.canRunResources(head) {
			s.readyQ = append(s.readyQ, head)
			s.inReady[head] = true
		}
	}
}

func uniqueResources(rs []Resource) []Resource {
	if len(rs) < 2 {
		return rs
	}
	seen := make(map[Resource]struct{}, len(rs))
	out := make([]Resource, 0, len(rs))
	for _, r := range rs {
		if _, ok := seen[r]; !ok {
			seen[r] = struct{}{}
			out = append(out, r)
		}
	}
	return out
}

func insertByPriority(q []*sTask, st *sTask) []*sTask {
	i := sort.Search(len(q), func(i int) bool {
		if q[i].Priority != st.Priority {
			return q[i].Priority < st.Priority
		}
		return q[i].seq > st.seq
	})
	q = append(q, nil)
	copy(q[i+1:], q[i:])
	q[i] = st
	return q
}

func removeTask(q []*sTask, st *sTask) []*sTask {
	for i, x := range q {
		if x == st {
			return append(q[:i], q[i+1:]...)
		}
	}
	return q
}

func safeRun(f func() error) (err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("scheduled task panicked: %v", r)
		}
	}()
	return f()
}

func CacheSlot(key string, slots int) Resource {
	if slots <= 1 {
		return "cache"
	}
	var h uint32 = 2166136261
	for i := 0; i < len(key); i++ {
		h = (h ^ uint32(key[i])) * 16777619
	}
	return Resource(fmt.Sprintf("cache-slot-%d", int(h%uint32(slots))))
}
