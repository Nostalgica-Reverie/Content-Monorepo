import { GuildMember } from 'discord.js'

export function isByActivatedUser(member: GuildMember): boolean {
	const activeRoleId = process.env.ACTIVE_ROLE_ID

	if (!activeRoleId) return false
	return member.roles.cache.has(activeRoleId)
}
