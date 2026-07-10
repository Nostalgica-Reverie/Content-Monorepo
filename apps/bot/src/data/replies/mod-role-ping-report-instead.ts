// MOD_PING_REPORT_INFO_URL: optional link to a message/page explaining how to
// report concerns; the "learn more" tail is omitted when unset.
const learnMore = process.env.MOD_PING_REPORT_INFO_URL
	? `, [learn more here](${process.env.MOD_PING_REPORT_INFO_URL})`
	: ''

export default `-# <:cornerdownright:${process.env.CORNER_DOWN_RIGHT_EMOJI_ID}> Please don't ping discord moderators, if you want to report a concern - either use ModMail or a new report feature${learnMore}.`
