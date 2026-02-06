run project example argument="":
  CARGO_TARGET_DIR=/tmp/dty/ RUST_LOG=trace cargo run -p {{project}} --example {{example}} {{argument}}

watch project example argument="":
  watchexec -rc clear CARGO_TARGET_DIR=/tmp/dty/ RUST_LOG=trace cargo run -p {{project}} --example {{example}} {{argument}}

generate-env:
  rm .env.defaults
  touch .env.defaults
  rm docs/docs/v1/config/env_config.md
  touch docs/docs/v1/config/env_config.md
  cat packages/app/config_template/app.env.defaults >>  .env.defaults
  echo "\n\n" >> .env.defaults
  cat packages/auth/config_template/auth.env.defaults >>  .env.defaults
  echo "\n\n" >> .env.defaults
  cat packages/cache/config_template/cache.env.defaults >>  .env.defaults
  echo "\n\n" >> .env.defaults
  cat packages/cron/config_template/cron.env.defaults >>  .env.defaults
  echo "\n\n" >> .env.defaults
  cat packages/db/config_template/db.env.defaults >>  .env.defaults
  echo "\n\n" >> .env.defaults
  cat packages/mail/config_template/mail.env.defaults >>  .env.defaults
  echo "\n\n" >> .env.defaults
  cat packages/multitenant/config_template/multitenant.env.defaults >>  .env.defaults
  echo "\n\n" >> .env.defaults
  cat packages/queue/config_template/queue.env.defaults >>  .env.defaults
  echo "\n\n" >> .env.defaults
  cat packages/session/config_template/session.env.defaults >>  .env.defaults
  echo "\n\n" >> .env.defaults
  cp .env.defaults bin/cli/src/stubs/.env.defaults.stub.txt
  echo "# Environment variables \n" >> docs/docs/v1/config/env_config.md;
  echo "\`\`\`ini" >> docs/docs/v1/config/env_config.md
  echo "\n" >> docs/docs/v1/config/env_config.md
  cat .env.defaults >> docs/docs/v1/config/env_config.md
  echo "\`\`\`" >> docs/docs/v1/config/env_config.md
  echo "\n" >> docs/docs/v1/config/env_config.md

build-api-doc:
  cargo doc --no-deps

