generate-end:
  rm .env.defaults
  touch .env.defaults
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



