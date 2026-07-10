import {
	ApplicationCommandType,
	ChatInputCommandInteraction,
	PermissionFlagsBits,
	SlashCommandBuilder,
} from 'discord.js'
import { eq } from 'drizzle-orm'

import { PERMISSION_ERROR_TEXT } from '@/data'
import { db } from '@/db'
import { watchlist } from '@/db/schema'
import { info } from '@/logging/logger'
import { ChatInputCommand } from '@/types'

export const watchlistCommand: ChatInputCommand = {
	type: ApplicationCommandType.ChatInput,
	data: new SlashCommandBuilder()
		.setName('watchlist')
		.setDescription(
			'Adds user to a watchlist, sends an alert when user joins the server with provided text.',
		)
		.setDefaultMemberPermissions(PermissionFlagsBits.BanMembers)
		.addStringOption((option) =>
			option.setName('id').setDescription('Discord User ID').setRequired(true),
		)
		.addStringOption((option) =>
			option.setName('alert_text').setDescription('Alert text').setRequired(true),
		) as SlashCommandBuilder,
	meta: {
		name: 'watchlist',
		description:
			'Adds user to a watchlist, sends an alert when user joins the server with provided text.',
		category: 'moderation',
		guildOnly: true,
	},
	execute: async (interaction: ChatInputCommandInteraction) => {
		if (!interaction.guild) return

		const id = interaction.options.getString('id', true)
		let member = null

		try {
			member = await interaction.guild.members.fetch(id)
		} catch {
			member = null
		}
		const alertText = interaction.options.getString('alert_text', true)

		const invoker = await interaction.guild.members.fetch(interaction.user.id)

		if (!invoker.roles.cache.has(process.env.DISCORD_MODERATOR_ROLE_ID!)) {
			await interaction.reply({
				content: PERMISSION_ERROR_TEXT,
				flags: 'Ephemeral',
			})
			return
		}

		const user = await db.select().from(watchlist).where(eq(watchlist.discordUserId, id)).limit(1)

		if (user[0]) {
			await db.delete(watchlist).where(eq(watchlist.discordUserId, id))

			await interaction.reply({
				content: `User has been already in the watchlist and was removed.`,
				flags: 'Ephemeral',
			})

			if (member) {
				info(
					`:no_bell: User ${member.user} (\`${member.user.username}\`, ID: ${member.user.id}) has been removed from the watchlist by moderator (\`${interaction.user.username}\`, ID: ${interaction.user.id}).`,
				)
			}
			return
		}

		if (member) {
			await interaction.reply({
				content: 'User is already on the server.',
				flags: 'Ephemeral',
			})
			return
		}

		await db.insert(watchlist).values({
			discordUserId: id,
			alertText: alertText,
		})

		await interaction.reply({
			content: `User has been added to the watchlist.`,
			flags: 'Ephemeral',
		})

		info(
			`:eye: User <@${id}> (ID: ${id}) has been added to the watchlist by moderator (\`${interaction.user.username}\`, ID: ${interaction.user.id}).`,
		)
	},
}
