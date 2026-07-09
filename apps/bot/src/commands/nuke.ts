import process from 'node:process'

import {
	ApplicationCommandType,
	ChatInputCommandInteraction,
	PermissionFlagsBits,
	SlashCommandBuilder,
} from 'discord.js'
import { eq } from 'drizzle-orm'
import { validate as uuidValidate } from 'uuid'

import { PERMISSION_ERROR_TEXT } from '@/data'
import { db } from '@/db'
import { applications } from '@/db/schema'
import { info } from '@/logging/logger'
import { ChatInputCommand } from '@/types'

export const nukeCommand: ChatInputCommand = {
	type: ApplicationCommandType.ChatInput,
	data: new SlashCommandBuilder()
		.setName('nuke')
		.setDescription('Delete an application entry from the database')
		.setDefaultMemberPermissions(PermissionFlagsBits.BanMembers)
		.addStringOption((option) =>
			option.setName('id').setDescription('Application ID').setRequired(true),
		) as SlashCommandBuilder,
	meta: {
		name: 'nuke',
		description: 'Delete an application entry from the database',
		category: 'moderation',
		cooldownSeconds: 3,
	},
	async execute(interaction: ChatInputCommandInteraction) {
		// Comma-separated Discord user IDs allowed to nuke applications (in
		// addition to needing the moderator role). Empty means nobody can.
		const LEAD_MODERATOR_IDS = (process.env.LEAD_MODERATOR_IDS ?? '')
			.split(',')
			.map((id) => id.trim())
			.filter(Boolean)

		if (!interaction.guild) return

		const invoker = await interaction.guild.members.fetch(interaction.user.id)

		const applicationId = interaction.options.getString('id', true)

		if (
			!invoker.roles.cache.has(process.env.DISCORD_MODERATOR_ROLE_ID!) ||
			!LEAD_MODERATOR_IDS.includes(invoker.user.id)
		) {
			await interaction.reply({
				content: PERMISSION_ERROR_TEXT,
				flags: 'Ephemeral',
			})
			return
		}

		if (!uuidValidate(applicationId)) {
			await interaction.reply({
				content: "You've provided an invalid UUID.",
				flags: 'Ephemeral',
			})
			return
		}

		const [linkedCase] = await db
			.select()
			.from(applications)
			.where(eq(applications.applicationId, applicationId))

		if (!linkedCase) {
			await interaction.reply({
				content: 'This application does not exist, have you typed it correctly?',
				flags: 'Ephemeral',
			})
			return
		}

		const [deleted] = await db
			.delete(applications)
			.where(eq(applications.applicationId, applicationId))
			.returning()

		await interaction.reply({
			content: `Successfully deleted application ${deleted.applicationId} sent by <@${deleted.userId}>.`,
			flags: 'Ephemeral',
		})

		info(
			`:boom: Trusted user application (ID: ${deleted.applicationId}) entry has been deleted by a moderator (\`${interaction.user.username}\`, ID: ${interaction.user.id}) from the database.`,
		)
	},
}
