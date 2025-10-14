# Issue: Implement Docker-based Local Development Environment

## Problem Statement

Currently, the project uses Neon (remote PostgreSQL database) for local development, which presents several issues:

- **Network dependency**: Development requires internet connectivity
- **Performance**: Network latency slows down development cycles
- **Cost**: Using paid database resources for development work
- **Isolation**: Risk of affecting shared development data
- **Team consistency**: Difficult to ensure all developers have identical environments

## Proposed Solution

Implement a Docker-based local development environment that includes:

### Services

- **PostgreSQL**: Local database instance
- **Redis**: For caching and session management
- **pgAdmin**: Database administration interface
- **Next.js App**: Containerized application

### Benefits

- **Offline development**: No internet dependency
- **Faster performance**: Local database access
- **Cost savings**: No paid database usage for development
- **Easy reset**: Fresh database with single command
- **Team consistency**: Identical environments via Docker
- **Isolation**: Complete separation from production data

## Implementation Details

### File Structure

```
src/nextjs/
├── Dockerfile.dev
├── docker-compose.dev.yml
├── .env.local.docker
└── scripts/
    ├── dev-setup.sh
    └── db/
        └── init/
            ├── 01-init-database.sql
            └── 02-seed-data.sql
```

### Services Configuration

- **PostgreSQL**: Port 5432, database `futura_dev`
- **Redis**: Port 6379
- **pgAdmin**: Port 5050 (admin@futura.local / admin)
- **Next.js**: Port 3000

### Development Workflow

```bash
# Start development environment
pnpm dev:docker

# Run migrations
pnpm db:docker:migrate

# Seed database
pnpm db:docker:seed

# Reset database
pnpm db:docker:reset
```

## Migration Strategy

1. **Phase 1**: Set up Docker environment alongside existing Neon setup
2. **Phase 2**: Migrate development team to Docker environment
3. **Phase 3**: Update documentation and onboarding
4. **Phase 4**: Remove Neon dependency for local development

## Acceptance Criteria

- [ ] Local development works completely offline
- [ ] Database can be reset to fresh state with single command
- [ ] All existing migrations and seed scripts work
- [ ] Team can switch between Neon and Docker environments
- [ ] Documentation updated with new setup instructions
- [ ] No impact on production or staging environments

## Priority

**Medium** - Improves developer experience and reduces costs, but not blocking current development.

## Authentication Considerations

Currently, development authentication uses real OAuth services (Google, GitHub, etc.) and creates real user accounts in the Neon database. This presents several issues:

- **Real OAuth calls**: Development hits actual OAuth providers
- **Rate limiting**: Risk of hitting OAuth provider limits
- **Data mixing**: Development users mixed with production data
- **Network dependency**: Authentication requires internet connectivity

### Auth.js Development Solutions

Auth.js (NextAuth) offers several options for local development authentication:

#### 1. Credentials Provider for Development

```javascript
// Add development-only credentials provider
if (process.env.NODE_ENV === "development") {
  providers.push({
    id: "dev",
    name: "Development Login",
    credentials: {
      email: { label: "Email", type: "email" },
      password: { label: "Password", type: "password" },
    },
    authorize: async (credentials) => {
      if (credentials.email === "dev@futura.local" && credentials.password === "dev123") {
        return {
          id: "dev-user-123",
          email: "dev@futura.local",
          name: "Development User",
        };
      }
      return null;
    },
  });
}
```

#### 2. Local OAuth Providers

- **Keycloak**: Full OAuth server for local development
- **Auth0 Local**: Auth0's local development server
- **Mock OAuth servers**: Lightweight OAuth implementations

#### 3. Benefits of Local Authentication

- **No external dependencies**: Work completely offline
- **Faster development**: No network calls to OAuth providers
- **Isolated testing**: No risk of affecting real user accounts
- **Consistent environment**: Same auth setup for all developers

## Technical Notes

- Docker files should be placed in `src/nextjs/` directory (not project root)
- Environment variables need to be configured for local Docker services
- Existing Drizzle migrations should work with local PostgreSQL
- Consider adding health checks for all services
- pgAdmin provides web interface for database management
- Auth.js development providers should be conditionally enabled
- Pre-seed database with test users for development

## S3 Storage Mocking

Currently, the project uses AWS S3 for file storage with extensive integration:

- **Direct uploads** via `uploadToS3()` function
- **Presigned URLs** for client-side uploads
- **AWS SDK v3** with `@aws-sdk/client-s3`
- **Environment-based configuration** with `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_S3_BUCKET`

### Mocking Options for Local Development

#### 1. LocalStack (Recommended)

```yaml
# Add to docker-compose.dev.yml
services:
  localstack:
    image: localstack/localstack
    ports:
      - "4566:4566"
    environment:
      - SERVICES=s3
      - DEBUG=1
```

**Benefits:**

- Complete AWS service emulator
- Existing S3 code works unchanged
- No code modifications needed
- Full S3 API compatibility

#### 2. MinIO (S3-compatible)

```yaml
services:
  minio:
    image: minio/minio
    ports:
      - "9000:9000"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
```

**Benefits:**

- S3-compatible API
- Lightweight alternative to LocalStack
- Good for simple S3 operations

#### 3. Environment-based Mocking

```javascript
// In development, replace S3 calls with local file storage
if (process.env.NODE_ENV === "development") {
  // Mock S3 upload - save to local filesystem
  return `http://localhost:3000/mock-files/${fileName}`;
}
```

**Benefits:**

- No additional Docker services
- Fastest for simple file operations
- Requires code modifications

### Recommended Approach

**LocalStack** is the best choice because:

- Zero code changes required
- Complete S3 API emulation
- Works with existing AWS SDK calls
- Supports all S3 features (presigned URLs, multipart uploads, etc.)

## Dependencies

- Docker and Docker Compose
- Existing Drizzle ORM setup
- Current database schema and migrations
- Auth.js configuration for development providers
- LocalStack for S3 mocking (recommended)
