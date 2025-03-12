
dev:
	cd frontend && pnpm dev &
	cd backend && cargo watch -x run
	|| pkill -9 webpack && pkill -9 cargo
