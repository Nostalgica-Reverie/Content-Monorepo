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
import { info } from '@/logging/logger'
import { ChatInputCommand } from '@/types'

export const resetCommand: ChatInputCommand = {
	type: ApplicationCommandType.ChatInput,
	data: new SlashCommandBuilder()
		.setName('reset')
		.setDescription("Remove user's trusted role with option to reset their message counter")
		.setDefaultMemberPermissions(PermissionFlagsBits.BanMembers)
		.addStringOption((option) =>
			option.setName('id').setDescription('Discord User ID').setRequired(true),
		)
		.addBooleanOption((option) =>
			option
				.setName('reset_counter')
				.setDescription("Reset user's message counter to 0")
				.setRequired(false),
		) as SlashCommandBuilder,
	meta: {
		name: 'reset',
		description: "Remove user's trusted role with option to reset their message counter",
		category: 'moderation',
		guildOnly: true,
	},
	execute: async (interaction: ChatInputCommandInteraction) => {
		if (!interaction.guild) return

		const id = interaction.options.getString('id', true)
		const member = await interaction.guild.members.fetch(id)
		const resetMessagesCounter = interaction.options.getBoolean('reset_counter', false)

		const invoker = await interaction.guild.members.fetch(interaction.user.id)

		if (!invoker.roles.cache.has(process.env.DISCORD_MODERATOR_ROLE_ID!)) {
			await interaction.reply({
				content: PERMISSION_ERROR_TEXT,
				flags: 'Ephemeral',
			})
			return
		}

		if (resetMessagesCounter)
			await db.update(users).set({ messagesSent: 0 }).where(eq(users.id, id))

		await member.roles.remove(process.env.TRUSTED_ROLE_ID!)

		await interaction.reply({
			content: `User's (\`${member.user.username}\`, ID: ${member.user.id}) trusted role and message counter has been reset.`,
			flags: 'Ephemeral',
		})
		info(
			`:pencil: User ${member.user} (\`${member.user.username}\`, ID: ${member.user.id}) has been reset by moderator (\`${interaction.user.username}\`, ID: ${interaction.user.id}).`,
		)
	},
}
