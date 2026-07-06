package murmur2

import "testing"

// Golden fingerprints for the CurseForge murmur2 variant (seed 1,
// whitespace stripped). Frozen as phase 4 characterization values.
func TestSum32GoldenVectors(t *testing.T) {
	vectors := map[string]uint32{
		"":                  1540447798,
		"packwiz":           2676380970,
		"helloworld":        2824650221,
		"hello \tworld\r\n": 2824650221,
	}
	for input, expected := range vectors {
		h := New()
		if _, err := h.Write([]byte(input)); err != nil {
			t.Fatal(err)
		}
		hash32, ok := h.(interface{ Sum32() uint32 })
		if !ok {
			t.Fatal("murmur2 hasher must expose Sum32")
		}
		if got := hash32.Sum32(); got != expected {
			t.Errorf("murmur2(%q) = %d, want %d", input, got, expected)
		}
	}
}

// Reset must clear the whitespace-stripped buffer completely.
func TestReset(t *testing.T) {
	h := New()
	h.Write([]byte("some earlier content"))
	h.Reset()
	h.Write([]byte("packwiz"))
	m, _ := h.(*Murmur2CF)
	if got := m.Sum32(); got != 2676380970 {
		t.Errorf("after Reset, murmur2(\"packwiz\") = %d, want 2676380970", got)
	}
}
