import { error } from '@/logging/logger'
import { CreateListener } from '@/types'
import { createDefaultEmbed, isInCommunitySupportThread } from '@/utils'

const logsLinkPattern = /https?:\/\/mclo\.gs\/([a-zA-Z0-9]+)/

async function getMcLogsInsights(logId: string) {
	const url = `https://api.mclo.gs/1/insights/${logId}`

	try {
		const response = await fetch(url)

		if (!response.ok) {
			error(`HTTP error! Status: ${response.status}`)
		}

		return await response.json()
	} catch (err) {
		error(`Failed to fetch mclo.gs insights: ${err}`)
		return null
	}
}

export const analyzeLogs: CreateListener = {
	id: 'forum:community-support:analyzeLogs',
	event: 'create',
	description:
		'Automatically analyze logs uploaded to mclo.gs in community support to send possible solutions',
	priority: 0,
	filter: { allowBots: false, allowDMs: false },
	match: async (ctx: { message: any }) => {
		if (!('message' in ctx)) return false
		const { message } = ctx
		if (!isInCommunitySupportThread(message)) return false
		return ctx.message.content?.match(logsLinkPattern) ?? false
	},
	handle: async (ctx) => {
		const regexMatch = ctx.message.content.match(logsLinkPattern)

		if (regexMatch) {
			const logId = regexMatch[1]

			getMcLogsInsights(logId).then(async (data) => {
				if (data.analysis.problems.length > 0) {
					let description = 'One or more common problems were detected, possible solutions:\n'

					data.analysis.problems.forEach((problem: any) => {
						if (problem.solutions.length > 0) {
							description += `\n**:warning: ${problem.message}**\nSolution: ${problem.solutions[0].message}`
						}
					})

					const embed = createDefaultEmbed()
						.setTitle(':white_check_mark: Possible solutions')
						.setDescription(description)
						.setFooter({ text: 'Powered by mclo.gs' })

					await ctx.message.reply({ embeds: [embed] })
				}
			})
		}
	},
}
