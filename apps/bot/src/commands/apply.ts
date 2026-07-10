import process from 'node:process'

import {
	ApplicationCommandType,
	ChatInputCommandInteraction,
	EmbedBuilder,
	SlashCommandBuilder,
	TextChannel,
} from 'discord.js'
import { and, desc, eq } from 'drizzle-orm'

import { db } from '@/db'
import { applications, users } from '@/db/schema'
import { info } from '@/logging/logger'
import { ChatInputCommand } from '@/types'
import { createDefaultEmbed } from '@/utils'

export const applyCommand: ChatInputCommand = {
	type: ApplicationCommandType.ChatInput,
	data: new SlashCommandBuilder()
		.setName('apply')
		.setDescription('Apply for a Advanced Trusted Programme'),
	meta: {
		name: 'apply',
		description: 'Apply for a Advanced Trusted Programme',
		category: 'utility',
		cooldownSeconds: 10,
	},
	async execute(interaction: ChatInputCommandInteraction) {
		if (!interaction.guild || !interaction.member || !interaction.inCachedGuild()) return

		if (interaction.member.roles.cache.has(process.env.TRUSTED_ROLE_ID!)) {
			await interaction.reply({
				content: "You're already an approved trusted user.",
				flags: 'Ephemeral',
			})
			return
		} else {
			const user = await db
				.select({
					id: users.id,
					messagesSent: users.messagesSent,
				})
				.from(users)
				.where(eq(users.id, interaction.user.id))
				.limit(1)

			const accountAgeMs = Date.now() - interaction.user.createdAt.getTime()
			const ninetyDaysMs = 90 * 24 * 60 * 60 * 1000
			const timestamp = Math.floor(Date.now() / 1000)

			if (!user[0]) {
				await interaction.reply({
					content: 'You do not meet our activity requirements yet.',
					flags: 'Ephemeral',
				})
				return
			}

			if (accountAgeMs < ninetyDaysMs || user[0].messagesSent < 100) {
				await interaction.user.send({
					embeds: [
						createDefaultEmbed({
							title: 'Trusted Role Application - Rejected',
							description: [
								'Your application has been rejected, please try again later.',
								' ',
								'Account does not matches our requirements, please be an active member of our community and try again later.',
							].join('\n'),
						}).setColor(0xff496e),
					],
				})
				await interaction.reply({
					content:
						'Check your direct messages, please ensure your DMs from this server are allowed to receive important messages.',
					flags: 'Ephemeral',
				})
				return
			} else {
				const pending = await db
					.select()
					.from(applications)
					.where(
						and(eq(applications.userId, interaction.user.id), eq(applications.status, 'pending')),
					)
					.limit(1)

				if (pending.length > 0) {
					await interaction.reply({
						content:
							':hourglass_flowing_sand: You already have a pending application, please be patient!',
						flags: 'Ephemeral',
					})
					return
				}

				const lastRejected = await db
					.select()
					.from(applications)
					.where(
						and(eq(applications.userId, interaction.user.id), eq(applications.status, 'rejected')),
					)
					.orderBy(desc(applications.createdAt))
					.limit(1)

				const rejected = lastRejected[0]

				if (rejected?.cooldownUntil && rejected.cooldownUntil.getTime() > Date.now()) {
					await interaction.reply({
						content:
							':no_entry_sign: You are temporary blocked from submitting new applications, try again later.',
						flags: 'Ephemeral',
					})
					return
				}

				const [created] = await db
					.insert(applications)
					.values({
						userId: interaction.user.id,
					})
					.returning()

				const channel = await interaction.guild.channels.fetch(process.env.APPLICATIONS_CHANNEL_ID!)

				if (channel && channel.isTextBased()) {
					const applicationEmbed = await (channel as TextChannel).send({
						embeds: [
							new EmbedBuilder({
								title: 'New Trusted Role Application - Pending',
								description: 'Received a new application for trusted role.',
								fields: [
									{
										name: 'User',
										value: `<@${created.userId}>`,
										inline: true,
									},
									{
										name: 'ID',
										value: created.userId,
										inline: true,
									},
									{
										name: 'Application ID',
										value: created.applicationId,
									},
									{
										name: 'Application Status',
										value: created.status,
									},
								],
								color: 0xffa347,
							}),
						],
					})

					await db
						.update(applications)
						.set({
							linkedMessageId: applicationEmbed.id,
						})
						.where(eq(applications.applicationId, created.applicationId))

					info(
						`:memo: New trusted role application has been created by <@${interaction.user.id}> (\`${interaction.user.username}\`, ID: ${interaction.user.id}).`,
					)
				}

				await interaction.user.send({
					embeds: [
						createDefaultEmbed({
							title: 'Trusted Role Application - In process',
							description: [
								`We're letting you know that application you've submitted on <t:${timestamp}:F> is being reviewed.`,
								' ',
								'Please be patient while your application is being reviewed, it can take up to 30 days or more for us to review it.',
								' ',
								'Note that this process is not automated and a real discord moderator is reviewing your details.',
							].join('\n'),
						}).setColor(0xffa347),
					],
				})

				await interaction.reply({
					content:
						'Check your direct messages, please ensure your DMs from this server are allowed to receive important messages.',
					flags: 'Ephemeral',
				})
			}
		}
	},
}
