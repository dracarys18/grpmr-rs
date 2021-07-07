<h1 align="center">TgBot-RS</h1>
<div align="center">
<a href="https://github.com/dracarys18/grpmr-rs/actions"><img src="https://github.com/dracarys18/grpmr-rs/actions/workflows/rust.yml/badge.svg?branch=master" width=100></a>
</div>
<p>This is a Telegram group manager bot written in rust. Some of the available features in this bot are:</p>
<p>
<li>
<code>/ban,/unban:</code> These commands ban/unban a user from a chat.
</li>
<li>
<code>/mute,/unmute:</code> This command will mute/unmute a user from a particular chat. 
</li>
<li>
<code>/tban &lt;time&gt;,/tmute &lt;time&gt; </code>: These commands will ban/mute a user from a chat until a given time.
</li>
<li>
<code>/kick:</code> Kicks a particular user from a chat.
</li>
<li>
<code>/info:</code> Gives info about a user Including his <code>first name,last name,user id,permanent url</code> of the user
</li>
<li>
<code>/id:</code> Gives user id if mentioned or just gives the id of the chat.
</li>
<li>
<code>/kickme:</code> Kicks the user who sent the command from the group
</li>
<li>
<code>/pin,/unpin:</code> Pins/Unpins the message in a chat.
</li>
<li>
<code>/promote,/demote:</code> Promotes a user to admin/Demotes the user and removes his admin permissions.
</li>
<li>
<code>/invitelink:</code> Sends the invitelink of the chat.
</li>
<li>
<code>/lock &lt;type&gt;,/unlock &lt;type&gt;:</code> Add or Remove some restrictions from the chat.
</li>
<li>
<code>/gban,/ungban:</code> Globally bans/unbans the user from the chats which are in common with the bot.
</li>
<li>
<code>/warn:</code> Warns the user in a chat when the warn count exceeds the limit the bot will kick/ban the user. 
</li>
<li>
<code>/ud &lt;word&gt;:</code> Find the meaning of a word in urban dictionary.
</li>
<li>
<code>/paste:</code> Pastes the given text into <a href='https://katb.in/'>Katbin</a> and sends the link of the paste.
</li>
<li>
<code>/disable &lt;command&gt;:</code> Disables the use of a command in a group.
</li>
<li>
<code>/filter &lt;keyword&gt; &lt;reply&gt;:</code> Enables a trigger on <code>keyword</code> and replies with <code>reply</code> whenever it matches with keyword. All <code>document,stickers,audio,video,photo</code> can be used as a trigger replies.
</li>
<li>
<code>Blacklist:</code> You can set any words as "blacklist" in your group and let the bot deal with whoever sends the blacklisted words automatically. The modes which are available currently are <code>Warn , Ban , Kick , Delete</code>
</li>
<li><code>Chat Methods:</code> You can set chat title, chat picture directly from the bot</li>
<li><code>Logging: </code> Recent actions are great but you can't see the changes that are older than 48 hours. So you can set-up a custom log channels to log the group properly and  access it whenever you want.</li>
<li><code>Reporting: </code> If you spot any suspicious activity in a group you can report that to admin by replying with /report it will send the report with the message that was reported to admins.</li>
</p>

<h1>How to Use?</h1>
<p>First off you would need to go to <code> @Botfather</code> in telegram and create a new bot and get the API token of the bot. And fill the  <code>TELOXIDE_TOKEN</code> in .env-example with the bot token. Fill <code>OWNER_ID</code> with your telegram user id and fill <code>SUDO_USERS</code> with the user id of your friends. Note that <code>SUDO_USERS</code> will have access to some of the admin commands in the groups which bot is in.</p>
<p>
Now go to <a href='https://www.mongodb.com/'>MongoDB</a> and create an instance and get the URI of your database. Paste the URI in <code>MONGO_URI</code>. Now rename <code>.env-example</code> to <code>.env</code> .
</p>
<p>
Now after all these are set-up to run the bot just execute
<code>
cargo run
</code>
from your terminal.
</p>
<h1>Credits</h1>
<li>
<a href='https://github.com/teloxide/teloxide'>teloxide</a> : The telegram framework bot uses.
</li>
<li>
<a href='https://github.com/PaulSonOfLars/tgbot'>MarieBot</a> : For the basic idea of the many of the features.
</li>