import { CronJob } from 'cron'
import { Client, IncidentActionsEditOptions } from 'discord.js'

export function startServerPauseDMs(client: Client) {
	const debug = process.env.CRON_DEBUG_MODE === '1'
	new CronJob(
		debug ? '*/10 * * * * *' : '0 * * * *', // run every hour on prod, every 10 sec on debug

		async function () {
			console.log('[Cron][ServerPauseDMs] Running cron job...')

			const guild = await client.guilds.fetch(process.env.GUILD_ID!)

			const current = guild.incidentsData?.dmsDisabledUntil

			// already enabled and not expired
			if (current && current.getTime() > Date.now()) {
				console.log('[Cron][ServerPauseDMs] DMs already paused')
				return
			}

			const incidentActions: IncidentActionsEditOptions = {
				dmsDisabledUntil: new Date(Date.now() + 86400000),
			}

			await client.guilds.setIncidentActions(process.env.GUILD_ID!, incidentActions)

			console.log('[Cron][ServerPauseDMs] DMs paused')
		},
		null,
		true,
		'America/Los_Angeles',
	)
}
