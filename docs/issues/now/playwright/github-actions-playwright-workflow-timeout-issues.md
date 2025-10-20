# GitHub Actions Playwright Workflow Timeout Issues

## 🚨 **Priority: High**

**Status:** In Progress  
**Assignee:** Tech Lead  
**Created:** $(date)  
**Labels:** `ci/cd`, `playwright`, `github-actions`, `blocking`

## 📋 **Summary**

The GitHub Actions Playwright workflow is experiencing timeout issues where the web server fails to start within the allocated time, causing all E2E tests to fail. This is blocking our ability to run automated tests in CI.

## 🔍 **Current Issue**

```
Error: Timed out waiting 120000ms from config.webServer.
Process completed with exit code 1.
```

The web server (`pnpm dev:nextjs`) is not starting successfully within the 2-minute timeout in the GitHub Actions environment.

## 🛠 **What We've Implemented**

### ✅ **Completed:**

1. **Internet Identity E2E Tests** - Complete test suite with proper test IDs
2. **Environment Variables** - Added required database and II environment variables
3. **Middleware Fix** - Added `PLAYWRIGHT=true` check to skip middleware during tests
4. **Caching** - Added proper caching for dependencies and Playwright browsers
5. **Timeout Increases** - Increased webServer timeout to 3 minutes, test timeout to 5 minutes

### 🔧 **Recent Changes:**

- Added server startup debugging step to identify the root cause
- Uncommented webServer configuration in `playwright.config.ts`
- Added proper environment variable handling at job level

## 🐛 **Root Cause Analysis**

### **Potential Issues:**

1. **Database Connection Timeout**

   - Server may be waiting for database connection
   - Database URL might be incorrect or unreachable in CI

2. **Environment Variable Issues**

   - Missing or incorrect environment variables
   - Variables not properly passed to web server process

3. **Port Conflicts**

   - Port 3000 might be in use
   - Server trying to use different port

4. **Dependencies/Installation Issues**

   - Node modules not properly installed
   - Missing system dependencies

5. **Next.js/Turbopack Issues**
   - Turbopack configuration conflicts
   - Build process hanging

## 📊 **Current Workflow Configuration**

```yaml
# .github/workflows/playwright.yml
env:
  DATABASE_URL: ${{ secrets.DATABASE_URL }}
  DATABASE_URL_UNPOOLED: ${{ secrets.DATABASE_URL_UNPOOLED }}
  AUTH_SECRET: ${{ secrets.AUTH_SECRET }}
  PLAYWRIGHT: true
  NEXT_PUBLIC_CANISTER_ID_INTERNET_IDENTITY: rdmx6-jaaaa-aaaaa-aaadq-cai
  NEXT_PUBLIC_II_URL: https://identity.ic0.app

webServer:
  command: "pnpm dev:nextjs"
  url: "http://localhost:3000"
  timeout: 180 * 1000 # 3 minutes
```

## 🔧 **Required Actions**

### **Immediate (This Week):**

1. **Verify GitHub Secrets**

   - [ ] Confirm `DATABASE_URL` secret is set correctly
   - [ ] Confirm `DATABASE_URL_UNPOOLED` secret is set correctly
   - [ ] Confirm `AUTH_SECRET` secret is set correctly

2. **Debug Server Startup**

   - [ ] Review the new debug step output in next CI run
   - [ ] Check if server is actually starting on port 3000
   - [ ] Verify database connectivity in CI environment

3. **Test Database Connection**
   - [ ] Ensure database is accessible from GitHub Actions
   - [ ] Check if database URL format is correct for CI
   - [ ] Verify database credentials are valid

### **Short Term (Next Sprint):**

4. **Alternative Solutions**

   - [ ] Consider using a test database for CI
   - [ ] Implement database seeding for tests
   - [ ] Add retry logic for server startup

5. **Monitoring & Logging**
   - [ ] Add comprehensive logging to server startup
   - [ ] Implement health checks for web server
   - [ ] Add metrics for CI performance

## 🧪 **Test Coverage Impact**

**Current Status:** ❌ **BLOCKED** - No E2E tests running in CI

**Tests Affected:**

- Internet Identity authentication flow (8 tests)
- User authentication (5 tests)
- Dashboard functionality (3 tests)
- Mobile responsiveness (6 tests)
- Account management (2 tests)
- **Total: 24 E2E tests not running**

## 📈 **Business Impact**

- **Quality Risk:** No automated E2E testing in CI
- **Development Velocity:** Manual testing required for all changes
- **Release Confidence:** Reduced confidence in deployments
- **Internet Identity:** Critical II authentication flow not tested automatically

## 🔗 **Related Files**

- `.github/workflows/playwright.yml` - Main workflow file
- `playwright.config.ts` - Playwright configuration
- `src/middleware.ts` - Middleware with Playwright skip logic
- `e2e/auth.internet-identity.spec.ts` - II tests
- `src/db/db.ts` - Database connection logic

## 📝 **Next Steps**

1. **Review debug output** from next CI run
2. **Verify all secrets** are properly configured
3. **Test database connectivity** from GitHub Actions
4. **Implement solution** based on root cause analysis
5. **Monitor CI performance** after fix

## 🎯 **Success Criteria**

- [ ] Playwright tests run successfully in GitHub Actions
- [ ] Web server starts within 3 minutes
- [ ] All 24 E2E tests pass
- [ ] Internet Identity tests complete successfully
- [ ] CI runs complete in under 10 minutes

## 📞 **Contact**

**Primary:** Tech Lead  
**Secondary:** DevOps Team  
**Escalation:** Engineering Manager

---

**Note:** This issue is blocking our ability to run automated E2E tests in CI, which is critical for maintaining code quality and release confidence. Priority should be given to resolving the web server startup issues.
