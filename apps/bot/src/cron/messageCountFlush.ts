import { CronJob } from 'cron'
import { sql } from 'drizzle-orm'

import { db } from '@/db'
import { users } from '@/db/schema'

// In-memory message-count deltas, keyed by user id. The count-messages
// listener increments this map instead of writing to Postgres per message;
// the cron below flushes accumulated deltas in one batched upsert. Up to one
// flush interval of counts can be lost on a hard crash — acceptable for an
// activity counter, and the flush re-credits the map if the write fails.
const pending = new Map<string, number>()

export function bufferMessageCount(userId: string) {
	pending.set(userId, (pending.get(userId) ?? 0) + 1)
}

export async function flushMessageCounts() {
	if (pending.size === 0) return
	const rows = [...pending.entries()].map(([id, delta]) => ({ id, messagesSent: delta }))
	pending.clear()
	try {
		await db
			.insert(users)
			.values(rows)
			.onConflictDoUpdate({
				target: users.id,
				set: { messagesSent: sql`${users.messagesSent} + excluded.messages_sent` },
			})
	} catch (err) {
		// Re-credit so the deltas survive until the next flush attempt.
		for (const row of rows) {
			pending.set(row.id, (pending.get(row.id) ?? 0) + row.messagesSent)
		}
		console.warn('[Cron][MessageCountFlush] Failed to flush message counts', err)
	}
}

export function startMessageCountFlushCron() {
	// Auto-starts (constructor start=true); no handle needed.
	new CronJob('*/30 * * * * *', flushMessageCounts, null, true, 'America/Los_Angeles')

	// Best-effort flush of whatever is buffered when the process is asked to
	// shut down, so a normal restart loses nothing.
	for (const signal of ['SIGINT', 'SIGTERM'] as const) {
		process.once(signal, () => {
			void flushMessageCounts().finally(() => process.exit(0))
		})
	}
}
