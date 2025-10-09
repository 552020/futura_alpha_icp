# Dashboard ICP/Neon Database Switching

**Priority**: Medium  
**Type**: Feature Implementation  
**Assigned To**: Dashboard Team  
**Created**: 2025-01-06  
**Status**: Open

## 🎯 Objective

Implement database switching functionality in the dashboard to allow users to view memories and folders from both ICP and Neon databases, with sync status indicators for advanced users.

## 📊 Current State

- ✅ **Backend**: Both ICP and Neon databases fully functional
- ✅ **Upload**: Users can choose ICP/Neon for database storage in settings
- ❌ **Dashboard**: Only shows memories from one database at a time
- ❌ **Sync Status**: No visibility into cross-database sync status

## 🔧 Required Features

### **1. Database Toggle Switch**

**Location**: Dashboard header/navigation  
**Functionality**: Toggle between "ICP Database" and "Neon Database" views

```typescript
interface DatabaseToggle {
  current: "icp" | "neon";
  available: ("icp" | "neon")[]; // Based on user's hosting preferences
  onToggle: (database: "icp" | "neon") => void;
}
```

### **2. Sync Status Indicators**

**For Advanced Users Only**: Show sync status between databases

- 🟢 **In Sync**: Memory exists in both databases
- 🟡 **Partial Sync**: Memory exists in one database only
- 🔴 **Out of Sync**: Memory exists in both but with different metadata

### **3. Memory Source Labels**

**Visual Indicators**: Show which database each memory is stored in

- `ICP` badge for ICP-only memories
- `Neon` badge for Neon-only memories
- `Both` badge for synced memories

## 🎨 UI/UX Requirements

### **Advanced Users Only**

- **Access Control**: Feature available only for users with advanced settings enabled
- **Settings Integration**: Respect user's database hosting preferences
- **Progressive Disclosure**: Hide sync indicators for basic users

### **Toggle Behavior**

- **Default View**: Show database from user's primary preference
- **Persistence**: Remember last selected database view
- **Fallback**: If only one database enabled, hide toggle

## 🔄 User Scenarios

### **Scenario 1: ICP-Only User**

- Toggle hidden (only ICP available)
- All memories show `ICP` badge

### **Scenario 2: Neon-Only User**

- Toggle hidden (only Neon available)
- All memories show `Neon` badge

### **Scenario 3: Dual Database User (Advanced)**

- Toggle visible with both options
- Sync status indicators shown
- Can switch between database views
- See which memories are synced vs. single-database

## 📋 Implementation Plan

### **Phase 1: Basic Toggle**

1. Add database toggle component to dashboard
2. Implement database switching logic
3. Update memory list to filter by selected database

### **Phase 2: Sync Status (Advanced Users)**

1. Add sync status API endpoints
2. Implement sync status indicators
3. Add memory source badges

### **Phase 3: Testing & Polish**

1. Test with various hosting preference combinations
2. Add loading states and error handling
3. Update documentation

## 🧪 Testing Requirements

- **Single Database**: Toggle hidden, memories display correctly
- **Dual Database**: Toggle functional, sync status accurate
- **Advanced Users**: Full feature set available
- **Basic Users**: Sync indicators hidden

## 📊 Success Metrics

1. **Functionality**: Users can view memories from both databases
2. **Sync Visibility**: Advanced users can see sync status
3. **Performance**: Database switching is fast and responsive
4. **UX**: Intuitive toggle behavior with clear visual indicators

## 🔗 Related Issues

- [Frontend ICP 2-Lane + 4-Asset Integration](./icp-413-wire-icp-memory-upload-frontend-backend/frontend-icp-2lane-4asset-integration.md)
- [Backend Bulk Memory APIs Implementation](./backend-bulk-memory-apis-implementation.md)

---

**Last Updated**: 2025-01-06  
**Status**: Open - Ready for Implementation  
**Priority**: Medium - Advanced User Feature
