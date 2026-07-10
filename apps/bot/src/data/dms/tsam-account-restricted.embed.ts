import { Colors, EmbedData } from 'discord.js'

// CONTACT_MENTION_ID: user/bot to mention for appeals (e.g. a ModMail bot).
const contact = process.env.CONTACT_MENTION_ID
	? `<@${process.env.CONTACT_MENTION_ID}>`
	: 'the staff team'

const data: EmbedData = {
	title: 'TSAM - Discord Account Restricted',
	description:
		'Your **Trust Score** has fallen below the required threshold, so your account has been temporarily restricted.\n\n' +
		'This action is automatically applied when our moderation system detects suspicious activity or potential rule violations.\n\n' +
		'While restricted, some server features and channels may be unavailable to you.\n\n' +
		`If you believe this action was applied in error, please contact us via ${contact}.`,
	footer: {
		text: 'Trust Score Auto Moderation',
	},
	color: Colors.Red,
}

export default data
