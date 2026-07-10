import { EmbedData } from 'discord.js'

const data: EmbedData = {
	description: [
		'**👋 Hello! Thank you for creating a new thread on our server**',
		' ',
		'📃 Something went wrong with the game? Make sure to provide logs using [mclo.gs](https://mclo.gs)',
		' ',
		`🔔 Don't forget to mark your thread as solved if issue has been resolved by using </solved:${process.env.SOLVED_COMMAND_ID}>`,
	].join('\n'),
}

export default data
