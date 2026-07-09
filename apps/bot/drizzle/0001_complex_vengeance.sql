CREATE TABLE "crowdin_accounts" (
	"discord_user_id" text PRIMARY KEY NOT NULL,
	"crowdin_user_id" text NOT NULL,
	"access_token" text NOT NULL,
	"refresh_token" text,
	"expires_at" timestamp with time zone NOT NULL,
	"organization_domain" text
);
--> statement-breakpoint
CREATE TABLE "oauth_verifications" (
	"token" text PRIMARY KEY NOT NULL,
	"provider" text NOT NULL,
	"discord_user_id" text NOT NULL,
	"expires_at" timestamp with time zone NOT NULL,
	"redirect_to" text
);
--> statement-breakpoint
CREATE INDEX "crowdin_accounts_user_idx" ON "crowdin_accounts" USING btree ("crowdin_user_id");--> statement-breakpoint
CREATE INDEX "oauth_verifications_provider_idx" ON "oauth_verifications" USING btree ("provider");