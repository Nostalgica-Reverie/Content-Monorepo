package cmd

import (
	"encoding/json"
	"io"
)

func jsonEncoder(writer io.Writer) *json.Encoder {
	encoder := json.NewEncoder(writer)
	encoder.SetEscapeHTML(true)
	return encoder
}
