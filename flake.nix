{
  description = "Entropy Engine - Rust trading CLI with PostgreSQL development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # Reference to your isolated development shells flake
    devshells.url = "path:/home/baso/.local/src/nixos-flakes/devshells";
  };

  outputs = { self, nixpkgs, flake-utils, devshells, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        
        # PostgreSQL configuration for development
        postgresql = pkgs.postgresql_15;
        
        # Redis configuration for development
        redis = pkgs.redis;
        
        # Database initialization script
        initdbScript = pkgs.writeShellScriptBin "init-entropy-db" ''
          set -e
          
          # Create .pg directory for local development database
          if [ ! -d ".pg" ]; then
            echo "🔧 Initializing PostgreSQL database for Entropy Engine..."
            mkdir -p .pg
            
            # Initialize database cluster
            ${postgresql}/bin/initdb -D .pg/data --locale=C --encoding=UTF8
            
            # Configure PostgreSQL for development
            cat >> .pg/data/postgresql.conf <<EOF
          # Development configuration for Entropy Engine
          port = 5555
          unix_socket_directories = '$PWD/.pg'
          log_statement = 'all'
          log_destination = 'stderr'
          logging_collector = on
          log_directory = 'log'
          log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
          EOF
            
            # Allow local connections without password for development
            cat > .pg/data/pg_hba.conf <<EOF
          # Development authentication for Entropy Engine
          local all all trust
          host all all 127.0.0.1/32 trust
          host all all ::1/128 trust
          EOF
            
            echo "✅ PostgreSQL initialized in .pg/ directory"
          else
            echo "📁 PostgreSQL already initialized"
          fi
          
          # Start PostgreSQL server
          echo "🚀 Starting PostgreSQL server on port 5555..."
          ${postgresql}/bin/pg_ctl -D .pg/data -l .pg/logfile start
          
          # Wait for server to start
          sleep 2
          
          # Create entropy_trading database if it doesn't exist
          if ! ${postgresql}/bin/psql -h localhost -p 5555 -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname='entropy_trading'" | grep -q 1; then
            echo "🗄️ Creating entropy_trading database..."
            ${postgresql}/bin/createdb -h localhost -p 5555 entropy_trading
            echo "✅ Database 'entropy_trading' created"
          else
            echo "📊 Database 'entropy_trading' already exists"
          fi
          
          echo ""
          echo "🎯 Entropy Engine PostgreSQL Development Environment Ready!"
          echo "📊 Database: entropy_trading"
          echo "🔌 Connection: postgresql://localhost:5555/entropy_trading"
          echo "⚡ SQLx CLI: sqlx --database-url postgresql://localhost:5555/entropy_trading"
          echo ""
        '';
        
        # Redis initialization script
        initRedisScript = pkgs.writeShellScriptBin "init-entropy-redis" ''
          set -e
          
          # Create .redis directory for local development cache
          if [ ! -d ".redis" ]; then
            echo "🔧 Initializing Redis cache for Entropy Engine..."
            mkdir -p .redis
            
            # Configure Redis for development
            cat > .redis/redis.conf <<EOF
# Development configuration for Entropy Engine
port 6379
bind 127.0.0.1
dir $PWD/.redis
dbfilename entropy_cache.rdb
logfile $PWD/.redis/redis.log
loglevel notice
save 900 1
save 300 10
save 60 10000
# Allow local connections for development
protected-mode no
EOF
            
            echo "✅ Redis initialized in .redis/ directory"
          else
            echo "📁 Redis already initialized"
          fi
          
          # Start Redis server
          echo "🚀 Starting Redis server on port 6379..."
          ${redis}/bin/redis-server .redis/redis.conf --daemonize yes
          
          # Wait for server to start
          sleep 2
          
          # Test connection
          if ${redis}/bin/redis-cli ping >/dev/null 2>&1; then
            echo "✅ Redis server started successfully"
          else
            echo "❌ Failed to start Redis server"
            exit 1
          fi
          
          echo ""
          echo "🎯 Entropy Engine Redis Development Environment Ready!"
          echo "🗄️ Cache: Redis on localhost:6379"
          echo "🔌 Connection: redis://localhost:6379"
          echo "⚡ Redis CLI: redis-cli"
          echo ""
        '';
        
        # Redis management scripts
        redisStartScript = pkgs.writeShellScriptBin "redis-start" ''
          if [ ! -f ".redis/redis.conf" ]; then
            echo "❌ Redis not initialized. Run 'init-entropy-redis' first."
            exit 1
          fi
          
          if ${redis}/bin/redis-cli ping >/dev/null 2>&1; then
            echo "✅ Redis is already running on port 6379"
          else
            echo "🚀 Starting Redis server..."
            ${redis}/bin/redis-server .redis/redis.conf --daemonize yes
            sleep 1
            if ${redis}/bin/redis-cli ping >/dev/null 2>&1; then
              echo "✅ Redis started successfully"
            else
              echo "❌ Failed to start Redis server"
              exit 1
            fi
          fi
          
          echo "🔗 REDIS_URL=redis://localhost:6379"
        '';
        
        redisStopScript = pkgs.writeShellScriptBin "redis-stop" ''
          if ${redis}/bin/redis-cli ping >/dev/null 2>&1; then
            echo "🛑 Stopping Redis server..."
            ${redis}/bin/redis-cli shutdown
            echo "✅ Redis stopped successfully"
          else
            echo "⚪ Redis is not running"
          fi
        '';
        
        redisStatusScript = pkgs.writeShellScriptBin "redis-status" ''
          if ${redis}/bin/redis-cli ping >/dev/null 2>&1; then
            echo "✅ Redis is running on port 6379"
            echo "🔗 REDIS_URL=redis://localhost:6379"
            echo "📊 Connect with: redis-cli"
            echo "💾 Info: $(${redis}/bin/redis-cli info server | grep redis_version)"
          else
            echo "⚪ Redis is not running"
            echo "💡 Start with: redis-start"
          fi
        '';
        
        redisConnectScript = pkgs.writeShellScriptBin "redis-connect" ''
          ${redis}/bin/redis-cli
        '';
        
        # Database management scripts
        pgStartScript = pkgs.writeShellScriptBin "pg-start" ''
          if [ ! -d ".pg/data" ]; then
            echo "❌ Database not initialized. Run 'init-entropy-db' first."
            exit 1
          fi
          
          if ${postgresql}/bin/pg_ctl -D .pg/data status >/dev/null 2>&1; then
            echo "✅ PostgreSQL is already running on port 5555"
          else
            echo "🚀 Starting PostgreSQL server..."
            ${postgresql}/bin/pg_ctl -D .pg/data -l .pg/logfile start
            echo "✅ PostgreSQL started successfully"
          fi
          
          echo "🔗 DATABASE_URL=postgresql://localhost:5555/entropy_trading"
        '';
        
        pgStopScript = pkgs.writeShellScriptBin "pg-stop" ''
          if ${postgresql}/bin/pg_ctl -D .pg/data status >/dev/null 2>&1; then
            echo "🛑 Stopping PostgreSQL server..."
            ${postgresql}/bin/pg_ctl -D .pg/data stop
            echo "✅ PostgreSQL stopped successfully"
          else
            echo "⚪ PostgreSQL is not running"
          fi
        '';
        
        pgStatusScript = pkgs.writeShellScriptBin "pg-status" ''
          if ${postgresql}/bin/pg_ctl -D .pg/data status >/dev/null 2>&1; then
            echo "✅ PostgreSQL is running on port 5555"
            echo "🔗 DATABASE_URL=postgresql://localhost:5555/entropy_trading"
            echo "📊 Connect with: psql postgresql://localhost:5555/entropy_trading"
          else
            echo "⚪ PostgreSQL is not running"
            echo "💡 Start with: pg-start"
          fi
        '';
        
        pgConnectScript = pkgs.writeShellScriptBin "pg-connect" ''
          ${postgresql}/bin/psql postgresql://localhost:5555/entropy_trading
        '';
        
        # SQLx CLI installation script with proper environment
        installSqlxScript = pkgs.writeShellScriptBin "install-sqlx-cli" ''
          set -e
          
          if command -v sqlx >/dev/null 2>&1; then
            echo "✅ SQLx CLI already installed: $(sqlx --version)"
            exit 0
          fi
          
          echo "📦 Installing SQLx CLI with PostgreSQL support..."
          echo "⏳ This may take several minutes on first install..."
          
          # Set environment for compilation
          export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
          export OPENSSL_DIR="${pkgs.openssl.dev}"
          export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib" 
          export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
          
          # Install with minimal features to reduce compilation time
          if cargo install sqlx-cli --no-default-features --features postgres,rustls,migrate 2>/dev/null; then
            echo "✅ SQLx CLI installed successfully: $(sqlx --version)"
          else
            echo "❌ SQLx CLI installation failed. You can install it manually later:"
            echo "   cargo install sqlx-cli --no-default-features --features postgres,rustls,migrate"
            echo ""
            echo "💡 The development environment will still work for building the project."
          fi
        '';
        
      in
      {
        devShells = {
          # Enhanced default shell with PostgreSQL support
          default = pkgs.mkShell {
            name = "entropy-engine-dev";
            
            buildInputs = with pkgs; [
              # Get Rust toolchain from devshells
              devshells.devShells.${system}.rust.buildInputs
              
              # PostgreSQL development dependencies
              postgresql
              
              # Redis development dependencies  
              redis
              
              # Note: SQLx CLI should be installed via cargo install sqlx-cli
              # Available after running: cargo install sqlx-cli --no-default-features --features postgres,rustls
              
              # Database management scripts
              initdbScript
              pgStartScript
              pgStopScript
              pgStatusScript
              pgConnectScript
              installSqlxScript
              
              # Redis management scripts
              initRedisScript
              redisStartScript
              redisStopScript
              redisStatusScript
              redisConnectScript
              
              # Development tools for SQLx compilation
              openssl
              openssl.dev
              pkg-config
              libiconv
              
              # Platform-specific dependencies for SQLx CLI
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.CoreFoundation  
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ] ++ [
              
              # Additional PostgreSQL tools
              postgresql.lib
              postgresql.dev
              
            ] ++ (devshells.devShells.${system}.rust.buildInputs or []);
            
            # Environment variables for development
            shellHook = ''
              # Check if SQLx CLI is available
              if ! command -v sqlx >/dev/null 2>&1; then
                echo "⚠️  SQLx CLI not found. Install with: install-sqlx-cli"
              else
                echo "✅ SQLx CLI available: $(sqlx --version)"
              fi
              
              export PGDATA="$PWD/.pg/data"
              export PGHOST="localhost"
              export PGPORT="5555"
              export PGUSER="$USER"
              export DATABASE_URL="postgresql://localhost:5555/entropy_trading"
              
              # Redis environment
              export REDIS_URL="redis://localhost:6379"
              
              # SQLx environment
              export SQLX_OFFLINE=true
              
              # Development tools
              export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
              
              # Add .pg directory to gitignore if not present
              if [ ! -f .gitignore ] || ! grep -q "^\.pg/$" .gitignore; then
                echo ".pg/" >> .gitignore
                echo "📝 Added .pg/ to .gitignore"
              fi
              
              # Add .redis directory to gitignore if not present  
              if [ ! -f .gitignore ] || ! grep -q "^\.redis/$" .gitignore; then
                echo ".redis/" >> .gitignore
                echo "📝 Added .redis/ to .gitignore"
              fi
              
              echo ""
              echo "🚀 Entropy Engine Development Environment"
              echo "========================================"
              echo "📊 PostgreSQL: Ready (use 'init-entropy-db' to initialize)"
              echo "🗄️ Redis: Ready (use 'init-entropy-redis' to initialize)"
              echo "⚡ SQLx CLI: Available"
              echo "🔧 Rust: $(rustc --version)"
              echo "🗄️ Database commands:"
              echo "  • init-entropy-db  - Initialize PostgreSQL database"
              echo "  • install-sqlx-cli - Install SQLx CLI for migrations"
              echo "  • pg-start        - Start PostgreSQL server"
              echo "  • pg-stop         - Stop PostgreSQL server"
              echo "  • pg-status       - Check PostgreSQL status"
              echo "  • pg-connect      - Connect to database"
              echo "🗄️ Redis commands:"
              echo "  • init-entropy-redis - Initialize Redis cache server"
              echo "  • redis-start       - Start Redis server"
              echo "  • redis-stop        - Stop Redis server"
              echo "  • redis-status      - Check Redis status"
              echo "  • redis-connect     - Connect to Redis CLI"
              echo ""
              echo "🔗 DATABASE_URL: $DATABASE_URL"
              echo "🔗 REDIS_URL: $REDIS_URL"
              echo "📝 Note: Database data stored in .pg/, cache data in .redis/ (both gitignored)"
              echo ""
            '';
            
            # Additional environment for PostgreSQL compilation
            env = {
              PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
              OPENSSL_DIR = "${pkgs.openssl.dev}";
              OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
              OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
            };
          };
          
          # Fallback to original Rust shell
          rust = devshells.devShells.${system}.rust;
        };
      });
}
