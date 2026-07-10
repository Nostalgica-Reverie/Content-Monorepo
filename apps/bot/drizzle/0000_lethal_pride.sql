CREATE TABLE "users" (
	"id" text PRIMARY KEY NOT NULL,
	"messages_sent" integer DEFAULT 0 NOT NULL,
	"crowdin_user_id" text,
	"modrinth_user_id" text
);
