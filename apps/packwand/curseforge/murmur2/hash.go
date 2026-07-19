package murmur2

import (
	"encoding/binary"
	"hash"
)

func New() hash.Hash32 {
	return &Murmur2CF{buf: make([]byte, 0)}
}

type Murmur2CF struct {
	// Can't be done incrementally, since it is seeded with the length of the input!
	buf []byte
}

func (m *Murmur2CF) Write(p []byte) (n int, err error) {
	for _, b := range p {
		if !isWhitespaceCharacter(b) {
			m.buf = append(m.buf, b)
		}
	}
	return len(p), nil
}

// CF modification: strips whitespace characters
func isWhitespaceCharacter(b byte) bool {
	return b == 9 || b == 10 || b == 13 || b == 32
}

func (m *Murmur2CF) Sum(b []byte) []byte {
	if b == nil {
		b = make([]byte, 4)
	}
	binary.BigEndian.PutUint32(b, MurmurHash2(m.buf, 1))
	return b
}

// MurmurHash2 is Austin Appleby's public-domain 32-bit MurmurHash2,
// implemented locally so the CurseForge fingerprint path does not depend on
// an unmaintained external module. Verified against the golden fingerprint
// vectors in hash_test.go.
func MurmurHash2(data []byte, seed uint32) uint32 {
	const m = 0x5bd1e995
	const r = 24
	h := seed ^ uint32(len(data))
	for len(data) >= 4 {
		k := binary.LittleEndian.Uint32(data)
		k *= m
		k ^= k >> r
		k *= m
		h *= m
		h ^= k
		data = data[4:]
	}
	switch len(data) {
	case 3:
		h ^= uint32(data[2]) << 16
		fallthrough
	case 2:
		h ^= uint32(data[1]) << 8
		fallthrough
	case 1:
		h ^= uint32(data[0])
		h *= m
	}
	h ^= h >> 13
	h *= m
	h ^= h >> 15
	return h
}

func (m *Murmur2CF) Reset() {
	m.buf = make([]byte, 0)
}

func (m *Murmur2CF) Size() int {
	return 4
}

func (m *Murmur2CF) BlockSize() int {
	return 4
}

func (m *Murmur2CF) Sum32() uint32 {
	return binary.BigEndian.Uint32(m.Sum(nil))
}
