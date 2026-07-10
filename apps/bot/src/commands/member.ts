import * as process from 'node:process'

import {
	ApplicationCommandType,
	ChatInputCommandInteraction,
	PermissionFlagsBits,
	SlashCommandBuilder,
} from 'discord.js'
import { eq } from 'drizzle-orm'

import { PERMISSION_ERROR_TEXT } from '@/data'
import { db } from '@/db'
import { users } from '@/db/schema'
import { ChatInputCommand } from '@/types'

export const memberCommand: ChatInputCommand = {
	type: ApplicationCommandType.ChatInput,
	data: new SlashCommandBuilder()
		.setName('member')
		.setDescription('Get internal information about discord user')
		.setDefaultMemberPermissions(PermissionFlagsBits.BanMembers)
		.addUserOption((option) =>
			option.setName('user').setDescription('Discord User').setRequired(true),
		) as SlashCommandBuilder,
	meta: {
		name: 'member',
		description: 'Get internal information about discord user',
		category: 'moderation',
		guildOnly: true,
	},
	execute: async (interaction: ChatInputCommandInteraction) => {
		if (!interaction.guild) return

		const user = interaction.options.getUser('user', true)

		const invoker = await interaction.guild.members.fetch(interaction.user.id)

		if (!invoker.roles.cache.has(process.env.DISCORD_MODERATOR_ROLE_ID!)) {
			await interaction.reply({
				content: PERMISSION_ERROR_TEXT,
				flags: 'Ephemeral',
			})
			return
		}

		const dbResponse = await db.select().from(users).where(eq(users.id, user.id))

		await interaction.reply({ content: JSON.stringify(dbResponse, null, 2), flags: 'Ephemeral' })
	},
}
