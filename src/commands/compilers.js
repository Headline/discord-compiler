import { Message, MessageEmbed, Client} from 'discord.js'
import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import DiscordMessageMenu from '../utils/DiscordMessageMenu'

export default class CompilersCommand extends CompilerCommand {
    /**
     *  Creates the Compile command
     */
    constructor(client) {
        super(client, {
            name: 'compilers',
            description: 'Displays the compilers for the specified language',
            developerOnly: false
        });
    }

    /**
     * Function which is executed when the command is requested by a user
     *
     * @param {CompilerCommandMessage} msg
     */
    async run(msg) {
        let args = msg.getArgs();
        if (args.length != 1) {
            msg.replyFail('You must supply a language in order view its supported compilers');
            return;
        }
        let langs = this.client.compilers.getCompilers(args[0].toLowerCase()); 
        if (!langs) {
            msg.replyFail(`The language *\'${args[0]}\'* is either not supported, or you have accidentially typed in the wrong language.` 
            + `Try using the *${this.client.prefix}languages* command to see supported languages!`);
            return;
        }
        let menu = new DiscordMessageMenu(msg.message, `Supported \'${args[0].toLowerCase()}\' compilers:`, 0x00FF00, 15);
        menu.buildMenu(langs);

        try {
            menu.displayPage(0);
        }
        catch (error) {
            msg.replyFail('Error with menu system, am I missing permissions?\n' + error);
        }
    }

    /**
     * Displays the help information for the given command
     *
     * @param {Message} grouper
     */
    async help(message) {

        response
            .setTitle('Command Usage')
            .setDescription(`*${this.description}*`)
            .addHelpField('Add a tag', `${this.toString()} add \`<tagName>\``)
            .addHelpField('Remove a tag', `${this.toString()} remove \`<tagName>\``)
            .isUsage()
        return message.dispatch(response);
    }
}