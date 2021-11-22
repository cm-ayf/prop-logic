import { ApplicationCommandData, Client, Intents } from "discord.js";
import { main } from "prop-logic";

const GUILD_ID = process.env.GUILD_ID ?? process.exit(1);
const BOT_TOKEN = process.env.BOT_TOKEN ?? process.exit(1);


const client = new Client({ intents: [
  Intents.FLAGS.GUILD_MESSAGES
]});

const commands: ApplicationCommandData[] = [
  {
    name: 'prop-logic',
    description: 'solves propositional logic (best effort)',
    options: [
      {
        name: 'input',
        description: 'logic to solve',
        type: 'STRING'
      },
      {
        name: 'tex',
        description: 'outputs in TeX',
        type: 'BOOLEAN',
        required: false,
      }
    ]
  },
];

client.on('ready', client => {
  console.log(`logged in as ${client.user.tag}`);

  client.guilds.fetch(GUILD_ID)
    .then(guild => guild.commands.set(commands))
    .then(() => console.log('command initialized.'), console.error);
})

client.on('interactionCreate', interaction => {
  if (!interaction.isCommand()) return;
  switch (interaction.commandName) {
    case 'prop-logic': {
      let input = interaction.options.getString('input');
      let tex = interaction.options.getBoolean('tex') ?? false;
      if (input) {
        let res = main(input, tex);
        interaction.reply(res);
      };
      break;
    }
    case 'help': {
      interaction.reply({
        content: 'https://github.com/cm-ayf/prop-logic',
        ephemeral: true
      });
      break;
    }
  }
});

process.on('SIGINT', async () => {
  await client.application?.commands.set([], GUILD_ID);
  process.exit(0);
});

client.login(BOT_TOKEN);