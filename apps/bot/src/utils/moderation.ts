import { GuildMember } from 'discord.js'

import { getTrustScore } from './trust'

const QUARANTINED_ROLE_ID = process.env.QUARANTINED_ROLE_ID!

export async function evaluateUser(member: GuildMember) {
	const trustScore = await getTrustScore(member.id)

	if (trustScore <= 10) {
		// very bad user
		await member.roles.add(QUARANTINED_ROLE_ID)
	} else if (trustScore <= 20) {
		// bad user
		await member.roles.add(QUARANTINED_ROLE_ID)
	} else if (trustScore <= 50) {
		// lower than a default score
	} else {
		// trusted user
		return
	}
}
