# Scriber Bot

This is a Discord bot project that provides session management and note-taking capabilities. It allows users to start sessions, end sessions, retrieve sessions, and retrieve notes for specific sessions within Discord channels.

## Installation

1. Clone the repository: `git clone https://github.com/ufe-pr/scriber-bot.git`
2. Navigate to the project directory: `cd scriber-bot`
3. Install dependencies: `cargo build`
4. Set up the necessary environment variables:
   - `DISCORD_TOKEN`: Discord bot token for authentication.
   - `SURREAL_DB_URL`: URL for the Surreal database.
   - `SURREAL_DB_USERNAME`: Username for Surreal database authentication (default: "root").
   - `SURREAL_DB_PASSWORD`: Password for Surreal database authentication (default: "root").
5. Run the bot: `cargo run`

## Usage

Once the bot is running and connected to the Discord server, you can use the following commands:

- `/start_session <name>`: Start a new session with the given name in the current channel.
- `/get_sessions`: Retrieve all active sessions in the current channel.
- `/end_session`: End the active session in the current channel.
- `/get_note`: Retrieve the notes for a specific session through an interaction menu.

## Project Structure

The project consists of the following files:

- `main.rs`: The main entry point of the application. It sets up the bot, handles events, and initializes the Surreal database.
- `commands.rs`: Defines the command functions that can be executed by users.
- `handlers.rs`: Contains the event handler function for processing new messages and creating notes.
- `notes.rs`: Provides functions for interacting with the notes data.
- `session.rs`: Implements session-related functions for managing active sessions.
- `utils.rs`: Contains utility functions used throughout the project.

## Dependencies

The project relies on the following external dependencies:

- `poise`: A framework for creating Discord bot commands.
- `surrealdb`: A Rust library for interacting with Surreal databases.
- `serenity`: A Discord library for the Rust programming language.
- `tokio`: An asynchronous runtime for Rust.

Please refer to the `Cargo.toml` file for the specific versions of these dependencies used in the project.

## Contributing

Contributions to this project are welcome! If you encounter any issues or have suggestions for improvements, please open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgments

- The [poise](https://github.com/serenity-rs/poise) and [serenity](https://github.com/serenity-rs/serenity) crates for their Discord bot framework and library.
- The [surrealdb](https://github.com/surrealdb/surrealdb) crate for interacting with Surreal databases.
