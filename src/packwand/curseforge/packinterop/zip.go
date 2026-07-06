package packinterop

import (
	"archive/zip"
	"errors"
	"strings"
)

type zipReaderFile struct {
	NameInternal string
	*zip.File
}

func (f zipReaderFile) Name() string {
	return f.NameInternal
}

type zipPackSource struct {
	MetaFile       *zip.File
	Reader         *zip.Reader
	cachedFileList []ImportPackFile
}

func (s *zipPackSource) updateFileList() {
	s.cachedFileList = make([]ImportPackFile, len(s.Reader.File))
	i := 0
	for _, v := range s.Reader.File {
		// Ignore directories
		if !v.Mode().IsDir() {
			// Some Windows tools (e.g. Compress-Archive) write zip entries
			// with backslash separators, against the zip spec; normalize so
			// the overrides/ prefix matching works on those packs too.
			s.cachedFileList[i] = zipReaderFile{strings.ReplaceAll(v.Name, "\\", "/"), v}
			i++
		}
	}
	s.cachedFileList = s.cachedFileList[:i]
}

func (s zipPackSource) GetFile(path string) (ImportPackFile, error) {
	for _, v := range s.cachedFileList {
		if v.Name() == path {
			return v, nil
		}
	}
	return zipReaderFile{}, errors.New("file not found in zip")
}

func (s zipPackSource) GetFileList() ([]ImportPackFile, error) {
	return s.cachedFileList, nil
}

func (s zipPackSource) GetPackFile() ImportPackFile {
	return zipReaderFile{s.MetaFile.Name, s.MetaFile}
}

func GetZipPackSource(metaFile *zip.File, reader *zip.Reader) ImportPackSource {
	source := zipPackSource{
		MetaFile: metaFile,
		Reader:   reader,
	}
	source.updateFileList()
	return source
}
