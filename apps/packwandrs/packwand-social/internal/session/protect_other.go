//go:build !windows

package session

func protect(data []byte) ([]byte, error) {
	return data, nil
}

func unprotect(data []byte) ([]byte, error) {
	return data, nil
}
