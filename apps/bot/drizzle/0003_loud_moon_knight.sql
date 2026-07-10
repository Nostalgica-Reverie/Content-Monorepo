CREATE TABLE "reports" (
	"report_id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"reported_user_id" text NOT NULL,
	"reporter_user_id" text,
	"message_id" text,
	"channel_id" text,
	"reason" text,
	"source" text NOT NULL,
	"automod_rule" text,
	"confidence_score" integer,
	"report_weight" integer,
	"reporter_trust_snapshot" integer,
	"status" text DEFAULT 'pending' NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"resolved_at" timestamp with time zone,
	"resolved_by" text,
	"evidence" jsonb
);
--> statement-breakpoint
CREATE INDEX "reports_reported_user_idx" ON "reports" USING btree ("reported_user_id");--> statement-breakpoint
CREATE INDEX "reports_reporter_user_idx" ON "reports" USING btree ("reporter_user_id");--> statement-breakpoint
CREATE INDEX "reports_status_idx" ON "reports" USING btree ("status");--> statement-breakpoint
CREATE INDEX "reports_created_idx" ON "reports" USING btree ("created_at");