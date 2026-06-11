package main

import (
	"fmt"
	"os"
	"sync"
	"time"
)

var sleepFrames = []string{"c(-.-)ɔ z  ", "c(-.-)ɔ zz ", "c(-.-)ɔ zzz", "c(-.-)ɔ  zz", "c(-.-)ɔ   z", "C(o.o)Ɔ !  "}

func isTTY() bool {
	fi, err := os.Stdout.Stat()
	if err != nil {
		return false
	}
	return fi.Mode()&os.ModeCharDevice != 0
}

type progress struct {
	mu     sync.Mutex
	label  string
	total  int
	n      int
	last   string
	tty    bool
	ticker *time.Ticker
	stop   chan struct{}
	frame  int
}

func newProgress(label string, total int) *progress {
	p := &progress{label: label, total: total, tty: isTTY()}
	if p.tty {
		p.stop = make(chan struct{})
		p.ticker = time.NewTicker(150 * time.Millisecond)
		go func() {
			for {
				select {
				case <-p.ticker.C:
					p.render()
				case <-p.stop:
					return
				}
			}
		}()
	}
	return p
}

func (p *progress) step(item string) {
	if !p.tty {
		return
	}
	p.mu.Lock()
	p.n++
	p.last = item
	p.mu.Unlock()
	p.render()
}

func (p *progress) render() {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.frame = (p.frame + 1) % len(sleepFrames)
	line := fmt.Sprintf("\r%s %s [%d/%d] %s", sleepFrames[p.frame], p.label, p.n, p.total, p.last)
	if len(line) > 100 {
		line = line[:100]
	}
	fmt.Printf("%-100s\r", line[1:])
}

func (p *progress) done() {
	if !p.tty {
		return
	}
	close(p.stop)
	p.ticker.Stop()
	fmt.Printf("\r%-100s\r", "")
}
