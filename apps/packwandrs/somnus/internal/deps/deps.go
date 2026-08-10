package deps

import "os/exec"

func Missing(names []string) []string {
	missing := make([]string, 0)
	for _, name := range names {
		if _, err := exec.LookPath(name); err != nil {
			missing = append(missing, name)
		}
	}
	return missing
}
