import type { Snowflake } from 'discord.js'
import { eq, sql } from 'drizzle-orm'

import { db } from '@/db'
import { users } from '@/db/schema'
import { info } from '@/logging/logger'

const DEFAULT_TRUST_SCORE = 50
const MIN_TRUST_SCORE = 0
const MAX_TRUST_SCORE = 100

/**
 * Get current trust score
 */
export async function getTrustScore(userId: Snowflake): Promise<number> {
	const user = await db.query.users.findFirst({
		where: eq(users.id, userId),
	})

	return user?.trustScore ?? DEFAULT_TRUST_SCORE
}

/**
 * Set trust score to absolute value
 */
export async function setTrustScore(userId: Snowflake, value: number): Promise<number> {
	const clampedValue = Math.max(MIN_TRUST_SCORE, Math.min(MAX_TRUST_SCORE, value))

	const result = await db
		.insert(users)
		.values({
			id: userId,
			trustScore: clampedValue,
		})
		.onConflictDoUpdate({
			target: users.id,
			set: { trustScore: clampedValue },
		})
		.returning({ trustScore: users.trustScore })

	info(`:shield: TSAM: Set trust score of <@${userId}> (ID: ${userId}) to ${result[0].trustScore}.`)
	return result[0].trustScore
}

/**
 * Reset trust score to default
 */
export async function resetTrustScore(userId: Snowflake): Promise<number> {
	info(`:shield: TSAM: Trust score of <@${userId}> (ID: ${userId}) was reset.`)
	return setTrustScore(userId, DEFAULT_TRUST_SCORE)
}

/**
 * Add trust score
 */
export async function addTrustScore(userId: Snowflake, amount: number): Promise<number> {
	const result = await db
		.insert(users)
		.values({
			id: userId,
			trustScore: DEFAULT_TRUST_SCORE,
		})
		.onConflictDoUpdate({
			target: users.id,
			set: {
				trustScore: sql`LEAST(${MAX_TRUST_SCORE}, GREATEST(${MIN_TRUST_SCORE}, ${users.trustScore} + ${amount}))`,
			},
		})
		.returning({ trustScore: users.trustScore })

	info(
		`:shield: TSAM: Updated trust score of <@${userId}> (ID: ${userId}) by ${amount}, current trust score is now ${result[0].trustScore}.`,
	)
	return result[0].trustScore
}

/**
 * Decrease trust score
 */
export async function decreaseTrustScore(userId: Snowflake, amount: number): Promise<number> {
	return addTrustScore(userId, -Math.abs(amount))
}
