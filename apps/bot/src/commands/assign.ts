import process from 'node:process'

import {
	ApplicationCommandType,
	ChatInputCommandInteraction,
	EmbedBuilder,
	PermissionFlagsBits,
	SlashCommandBuilder,
} from 'discord.js'
import { eq } from 'drizzle-orm'
import { validate as uuidValidate } from 'uuid'

import { PERMISSION_ERROR_TEXT } from '@/data'
import { db } from '@/db'
import { applications } from '@/db/schema'
import { ChatInputCommand } from '@/types'

export const assignCommand: ChatInputCommand = {
	type: ApplicationCommandType.ChatInput,
	data: new SlashCommandBuilder()
		.setName('assign')
		.setDescription('Assign case to a moderator')
		.setDefaultMemberPermissions(PermissionFlagsBits.BanMembers)
		.addStringOption((option) =>
			option.setName('id').setDescription('Application ID').setRequired(true),
		)
		.addUserOption((option) =>
			option.setName('moderator').setDescription('Discord Moderator').setRequired(false),
		) as SlashCommandBuilder,
	meta: {
		name: 'assign',
		description: 'Assign case to a moderator',
		category: 'moderation',
		cooldownSeconds: 3,
	},
	async execute(interaction: ChatInputCommandInteraction) {
		if (!interaction.guild) return

		const invoker = await interaction.guild.members.fetch(interaction.user.id)

		const applicationId = interaction.options.getString('id', true)
		const discordModerator = interaction.options.getUser('moderator', false) ?? invoker.user

		if (!invoker.roles.cache.has(process.env.DISCORD_MODERATOR_ROLE_ID!)) {
			await interaction.reply({
				content: PERMISSION_ERROR_TEXT,
				flags: 'Ephemeral',
			})
			return
		}

		if (discordModerator?.bot) {
			await interaction.reply({
				content: 'You just tried to assign an application to a bot, lazy.',
				flags: 'Ephemeral',
			})
			return
		}

		if (
			!(await interaction.guild.members.fetch(discordModerator.id)).roles.cache.has(
				process.env.DISCORD_MODERATOR_ROLE_ID!,
			)
		) {
			await interaction.reply({
				content: "You've attempted to assign a user that's not a moderator member.",
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
			.where(eq(applications.applicationId, applicationId!))

		if (!linkedCase) {
			await interaction.reply({
				content: 'This application does not exist, have you typed it correctly?',
				flags: 'Ephemeral',
			})
			return
		}

		if (linkedCase.reviewedBy) {
			await interaction.reply({
				content: `This application is already assigned to <@${linkedCase.reviewedBy}>.`,
				flags: 'Ephemeral',
			})
			return
		}

		if (linkedCase.status != 'pending') {
			await interaction.reply({
				content: 'This application has already been reviewed.',
				flags: 'Ephemeral',
			})
			return
		}

		const [updated] = await db
			.update(applications)
			.set({
				reviewedBy: discordModerator!.id,
			})
			.where(eq(applications.applicationId, applicationId!))
			.returning()

		const applicationsChannel = await interaction.guild.channels.fetch(
			process.env.APPLICATIONS_CHANNEL_ID!,
		)

		if (applicationsChannel && applicationsChannel.isTextBased()) {
			const linkedApplicationEmbed = await applicationsChannel.messages.fetch(
				updated.linkedMessageId!,
			)

			const embed = EmbedBuilder.from(linkedApplicationEmbed.embeds[0])

			await linkedApplicationEmbed.edit({
				embeds: [
					embed.setDescription(`**Application is being reviewed by <@${updated.reviewedBy}>**`),
				],
			})
		}

		await interaction.reply({
			content: `Successfully assigned moderator <@${updated.reviewedBy}> to application.`,
			flags: 'Ephemeral',
		})
	},
}
