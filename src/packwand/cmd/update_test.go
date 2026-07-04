package cmd

import "testing"

func TestUpdateFailureError(t *testing.T) {
	if err := updateFailureError(nil); err != nil {
		t.Fatalf("nil report: %v", err)
	}
	report := newUpdateReport()
	if err := updateFailureError(report); err != nil {
		t.Fatalf("empty report: %v", err)
	}
	report.Failed = append(report.Failed, updateReportEntry{Name: "broken", Error: "network failure"})
	if err := updateFailureError(report); err == nil {
		t.Fatal("expected a failed update report to return an error")
	}
}
