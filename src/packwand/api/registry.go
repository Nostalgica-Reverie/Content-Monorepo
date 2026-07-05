package api

import (
	"fmt"
	"net/http"
	"reflect"
	"sort"
	"sync"
)

// Action describes one command exposed by the HTTP API. The same registry is
// used for routing, capability discovery, and OpenAPI generation.
type Action struct {
	Name        string
	Method      string
	Path        string
	Summary     string
	Destructive bool
	Build       func(*Server, *http.Request) (dir string, argv []string, err error)
	Result      reflect.Type
}

var actionRegistry = struct {
	sync.RWMutex
	actions map[string]Action
}{actions: map[string]Action{}}

// Register adds an API action. Duplicate names or method/path pairs panic at
// startup because either would make the public contract ambiguous.
func Register(action Action) {
	if action.Name == "" || action.Method == "" || action.Path == "" || action.Build == nil {
		panic("api: incomplete action registration")
	}
	actionRegistry.Lock()
	defer actionRegistry.Unlock()
	if _, exists := actionRegistry.actions[action.Name]; exists {
		panic("api: duplicate action " + action.Name)
	}
	for _, current := range actionRegistry.actions {
		if current.Method == action.Method && current.Path == action.Path {
			panic(fmt.Sprintf("api: duplicate route %s %s", action.Method, action.Path))
		}
	}
	actionRegistry.actions[action.Name] = action
}

func actions() []Action {
	actionRegistry.RLock()
	defer actionRegistry.RUnlock()
	out := make([]Action, 0, len(actionRegistry.actions))
	for _, action := range actionRegistry.actions {
		out = append(out, action)
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Name < out[j].Name })
	return out
}

func actionByName(name string) (Action, bool) {
	actionRegistry.RLock()
	defer actionRegistry.RUnlock()
	action, ok := actionRegistry.actions[name]
	return action, ok
}
