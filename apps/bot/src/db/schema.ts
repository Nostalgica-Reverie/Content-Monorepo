import { index, integer, jsonb, pgTable, text, timestamp, uuid } from 'drizzle-orm/pg-core'

export const users = pgTable('users', {
	id: text('id').primaryKey(),
	messagesSent: integer('messages_sent').notNull().default(0),
	trustScore: integer('trust_score').notNull().default(50),
	crowdinUserId: text('crowdin_user_id'),
})

export const watchlist = pgTable('watchlist', {
	discordUserId: text('discord_user_id').primaryKey().unique(),
	alertText: text('alert_text').notNull(),
})

export const reports = pgTable(
	'reports',
	{
		reportId: uuid('report_id').primaryKey().defaultRandom(),

		reportedUserId: text('reported_user_id').notNull(),
		reporterUserId: text('reporter_user_id'),

		messageId: text('message_id'),
		channelId: text('channel_id'),

		reason: text('reason'),

		source: text('source').notNull(), // user | automod
		automodRule: text('automod_rule'),

		confidenceScore: integer('confidence_score'),
		reportWeight: integer('report_weight'),
		reporterTrustSnapshot: integer('reporter_trust_snapshot'),

		status: text('status').notNull().default('pending'),

		createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow(),

		resolvedAt: timestamp('resolved_at', { withTimezone: true }),
		resolvedBy: text('resolved_by'),

		evidence: jsonb('evidence'),
	},
	(table) => ({
		reportedUserIdx: index('reports_reported_user_idx').on(table.reportedUserId),
		reporterUserIdx: index('reports_reporter_user_idx').on(table.reporterUserId),
		statusIdx: index('reports_status_idx').on(table.status),
		createdIdx: index('reports_created_idx').on(table.createdAt),
	}),
)

export const applications = pgTable(
	'applications',
	{
		applicationId: uuid('application_id').primaryKey().defaultRandom(),

		userId: text('user_id').notNull(),

		status: text('status').notNull().default('pending'),
		// pending | approved | rejected

		createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow(),

		reviewedAt: timestamp('reviewed_at', { withTimezone: true }),

		reviewedBy: text('reviewed_by'),

		rejectionReason: text('rejection_reason'),

		cooldownUntil: timestamp('cooldown_until', {
			withTimezone: true,
		}),

		linkedMessageId: text('linked_message_id'),
	},
	(table) => ({
		userIdx: index('applications_user_idx').on(table.userId),
		statusIdx: index('applications_status_idx').on(table.status),
		createdIdx: index('applications_created_idx').on(table.createdAt),
	}),
)

export type User = typeof users.$inferSelect
export type NewUser = typeof users.$inferInsert
export type Report = typeof reports.$inferSelect
export type NewReport = typeof reports.$inferInsert

// Stores short-lived verification state tokens for OAuth CSRF protection
export const oauthVerifications = pgTable(
	'oauth_verifications',
	{
		token: text('token').primaryKey(),
		provider: text('provider').notNull(),
		discordUserId: text('discord_user_id').notNull(),
		expiresAt: timestamp('expires_at', { withTimezone: true }).notNull(),
		redirectTo: text('redirect_to'),
	},
	(t) => ({ byProvider: index('oauth_verifications_provider_idx').on(t.provider) }),
)

// Stores Crowdin OAuth tokens mapped to Discord and Crowdin identities
export const crowdinAccounts = pgTable(
	'crowdin_accounts',
	{
		discordUserId: text('discord_user_id').primaryKey(),
		crowdinUserId: text('crowdin_user_id').notNull(),
		accessToken: text('access_token').notNull(),
		refreshToken: text('refresh_token'),
		expiresAt: timestamp('expires_at', { withTimezone: true }).notNull(),
		// enterprise org domain if present
		organizationDomain: text('organization_domain'),
	},
	(t) => ({ byCrowdin: index('crowdin_accounts_user_idx').on(t.crowdinUserId) }),
)

export type CrowdinAccount = typeof crowdinAccounts.$inferSelect
export type NewCrowdinAccount = typeof crowdinAccounts.$inferInsert
