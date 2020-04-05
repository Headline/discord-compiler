import { Client, Guild, MessageEmbed } from 'discord.js'

/**
 * A helper class which abstracts all support server information postings. 
 */
export default class SupportServer {
    /**
     * Creates a SupportServer object & sets necessary instance variables for proper function
     * 
     * @param {Client} client 
     */
    constructor(client) {
        let guild = null;
         client.guilds.cache.forEach((g) => {
            if (g.id == this.client.support_server) {
                guild = g;
            }
        })
        
        this.supportguild = guild;
    }

    /**
     * Posts to the join log of the support server for tracking.
     * 
     * @param {Guild} guild
     */
    postJoined(guild)
    {
        if (this.supportguild == null)
            return;

        let channel = null;
        this.supportguild.channels.cache.forEach((c) => {
            if (c.name === "join-log")
                channel = c;
        });

        if (channel == null)
            return;

        const embed = new MessageEmbed()
        .setThumbnail(guild.iconURL)
        .setTitle('Server Joined:')    
        .setColor(0x00FF00)
        .addField("Name", guild.name, true)
        .addField("Guild Id",  guild.id, true)
        .addField("Total Members", guild.memberCount, true)
        .addField("Total Channels", guild.channels.array.length, true)
        .addField("Guild Owner", guild.owner.user.tag, true)
        .addField("Guild Region", guild.region, true)
        .addField("Creation Date", guild.createdAt.toISOString(), true)
        channel.send(embed).catch();
    }

    /**
     * Posts to the join log of the support server for tracking.
     * 
     * @param {Guild} guild
     */
    postLeft(guild)
    {
        if (this.supportguild == null)
            return;

        let channel = null;
        
        this.supportguild.channels.forEach((element) => {
            if (element.name === "join-log")
                channel = element;
        });

        if (channel == null)
            return;

        const embed = new MessageEmbed()
        .setThumbnail(guild.iconURL)
        .setTitle('Server Left:')    
        .setColor(0xFF0000)
        .addField("Name", guild.name, true)
        .addField("Guild Id",  guild.id, true)
        .addField("Total Members", guild.memberCount, true)
        .addField("Total Channels", guild.channels.array.length, true)
        .addField("Guild Owner", guild.owner.user.tag, true)
        .addField("Guild Region", guild.region, true)
        .addField("Creation Date", guild.createdAt.toISOString(), true)
        channel.send(embed).catch(console.log);
    }

    postCompilation(code, lang, url, author, guild, success, failoutput) {
        if (this.supportguild == null)
            return;

        let channel = null;

        this.supportguild.channels.forEach((element) => {
        if (element.name === "compile-log")
            channel = element;
        });

        if (channel == null)
            return;

        if (code.length >= 1017) {
            code = code.substring(0, 1016);
        }
        const embed = new MessageEmbed()
        .setTitle('Compilation Requested:')    
        .setColor((success)?0x00FF00:0xFF0000)
        .addField("Language", lang, true)
        .addField("URL",  url, true)
        .addField("User",  author.tag, true)
        .addField("Guild",  guild.name, true)
        .addField('Code', `\`\`\`${code}\n\`\`\`\n`);
        if (!success)
            embed.addField('Compiler Output', `\`\`\`${failoutput}\n\`\`\`\n`);
        channel.send(embed).catch(console.log);
    }
}