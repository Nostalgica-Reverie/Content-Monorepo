package main

import (
	"io"
	"os"
)

const copyBufSize = 1 << 20

func copyFileFast(src, dst string) error {
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()

	out, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer out.Close()

	buf := make([]byte, copyBufSize)
	_, err = io.CopyBuffer(out, in, buf)
	return err
}
