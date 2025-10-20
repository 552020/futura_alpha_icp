# Memory Sharing Modal Implementation

## Problem Description

Currently, the memory sharing functionality is not implemented. Users need a way to share individual memories with other users through:

- Contact selection (from user's contact list)
- Email address input
- Public link generation

The sharing system should integrate with the existing `resourceMembership` table and support both private sharing (specific users) and public sharing (anyone with link).

## Current State Analysis

### Existing Infrastructure

- **Database**: `resourceMembership` table exists for tracking shared resources
- **API**: Memory sharing status is calculated in `/api/memories` endpoint
- **UI**: `ContentCard` has `onShare` prop but no implementation
- **Types**: `Memory` type includes `sharingStatus` and `sharedWithCount`

### Missing Components

- **Sharing Modal**: No UI component for sharing interface
- **Contact System**: No contact management functionality
- **Share API**: No endpoints for creating/updating shares
- **Link Generation**: No public link creation system

## Proposed Solution

### Core Features

1. **Share Modal**: Reusable modal for memory sharing
2. **Contact Selection**: Choose from user's contacts (future feature)
3. **Email Sharing**: Share with specific email addresses
4. **Public Links**: Generate shareable public links
5. **Permission Management**: View and revoke existing shares

### Technical Architecture

#### 1. Sharing Modal Component

```typescript
interface MemoryShareModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  memory: Memory;
  onShare: (shareData: ShareData) => Promise<void>;
}

interface ShareData {
  type: "email" | "public_link" | "contact";
  recipients?: string[];
  permissions?: SharePermissions;
  expiresAt?: Date;
}
```

#### 2. API Endpoints

- `POST /api/memories/[id]/share` - Create new share
- `GET /api/memories/[id]/shares` - List existing shares
- `DELETE /api/memories/[id]/shares/[shareId]` - Revoke share
- `POST /api/memories/[id]/public-link` - Generate public link

#### 3. Database Schema Extensions

```sql
-- New table for public links
CREATE TABLE memory_public_links (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  memory_id UUID REFERENCES memories(id) ON DELETE CASCADE,
  token VARCHAR(255) UNIQUE NOT NULL,
  created_by UUID REFERENCES all_users(id),
  expires_at TIMESTAMP,
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMP DEFAULT NOW()
);

-- Index for token lookup
CREATE INDEX idx_memory_public_links_token ON memory_public_links(token);
```

## Implementation Plan

### Phase 1: Core Sharing Modal

**Duration**: 3-4 hours

#### 1.1 Modal Component Structure

```typescript
// src/components/memory/memory-share-modal.tsx
export function MemoryShareModal({ open, onOpenChange, memory, onShare }: MemoryShareModalProps) {
  const [shareType, setShareType] = useState<"email" | "public_link">("email");
  const [emailAddresses, setEmailAddresses] = useState<string[]>([]);
  const [permissions, setPermissions] = useState<SharePermissions>("view");

  // Modal tabs: Email, Public Link, Existing Shares
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Share Memory</DialogTitle>
          <DialogDescription>Share "{memory.title}" with others</DialogDescription>
        </DialogHeader>

        <Tabs defaultValue="email" className="w-full">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="email">Email</TabsTrigger>
            <TabsTrigger value="public">Public Link</TabsTrigger>
            <TabsTrigger value="existing">Existing Shares</TabsTrigger>
          </TabsList>

          <TabsContent value="email">
            <EmailShareForm />
          </TabsContent>

          <TabsContent value="public">
            <PublicLinkForm />
          </TabsContent>

          <TabsContent value="existing">
            <ExistingSharesList />
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
```

#### 1.2 Email Sharing Form

```typescript
function EmailShareForm() {
  const [emails, setEmails] = useState<string[]>([]);
  const [newEmail, setNewEmail] = useState("");
  const [permissions, setPermissions] = useState<"view" | "edit">("view");

  return (
    <div className="space-y-4">
      <div>
        <Label>Email Addresses</Label>
        <div className="flex gap-2">
          <Input
            placeholder="Enter email address"
            value={newEmail}
            onChange={(e) => setNewEmail(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && newEmail) {
                setEmails([...emails, newEmail]);
                setNewEmail("");
              }
            }}
          />
          <Button onClick={addEmail}>Add</Button>
        </div>

        {/* Email chips */}
        <div className="flex flex-wrap gap-2 mt-2">
          {emails.map((email, index) => (
            <Badge key={index} variant="secondary" className="flex items-center gap-1">
              {email}
              <Button size="sm" variant="ghost" onClick={() => removeEmail(index)}>
                <X className="h-3 w-3" />
              </Button>
            </Badge>
          ))}
        </div>
      </div>

      <div>
        <Label>Permissions</Label>
        <RadioGroup value={permissions} onValueChange={setPermissions}>
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="view" id="view" />
            <Label htmlFor="view">View only</Label>
          </div>
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="edit" id="edit" />
            <Label htmlFor="edit">Can edit</Label>
          </div>
        </RadioGroup>
      </div>
    </div>
  );
}
```

#### 1.3 Public Link Form

```typescript
function PublicLinkForm() {
  const [linkGenerated, setLinkGenerated] = useState(false);
  const [publicLink, setPublicLink] = useState("");
  const [expiresAt, setExpiresAt] = useState<Date | null>(null);

  const generateLink = async () => {
    // Call API to generate public link
    const response = await fetch(`/api/memories/${memory.id}/public-link`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ expiresAt }),
    });

    const data = await response.json();
    setPublicLink(data.link);
    setLinkGenerated(true);
  };

  return (
    <div className="space-y-4">
      {!linkGenerated ? (
        <>
          <div>
            <Label>Link Expiration (Optional)</Label>
            <Input
              type="datetime-local"
              value={expiresAt ? expiresAt.toISOString().slice(0, 16) : ""}
              onChange={(e) => setExpiresAt(e.target.value ? new Date(e.target.value) : null)}
            />
          </div>

          <Button onClick={generateLink} className="w-full">
            Generate Public Link
          </Button>
        </>
      ) : (
        <div className="space-y-4">
          <div>
            <Label>Public Link</Label>
            <div className="flex gap-2">
              <Input value={publicLink} readOnly />
              <Button size="sm" onClick={() => navigator.clipboard.writeText(publicLink)}>
                <Copy className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>Anyone with this link can view the memory. Keep it secure.</AlertDescription>
          </Alert>
        </div>
      )}
    </div>
  );
}
```

### Phase 2: API Implementation

**Duration**: 4-5 hours

#### 2.1 Share Creation API

```typescript
// src/app/api/memories/[id]/share/route.ts
export async function POST(request: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  const session = await auth();
  if (!session?.user?.id) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  const { id: memoryId } = await params;
  const body = await request.json();
  const { type, recipients, permissions, expiresAt } = body;

  // Validate memory ownership
  const memory = await db.query.memories.findFirst({
    where: and(eq(memories.id, memoryId), eq(memories.ownerId, allUserRecord.id)),
  });

  if (!memory) {
    return NextResponse.json({ error: "Memory not found" }, { status: 404 });
  }

  if (type === "email") {
    // Create resourceMembership entries for each email
    const memberships = await Promise.all(
      recipients.map(async (email: string) => {
        // Find or create user by email
        const targetUser = await findOrCreateUserByEmail(email);

        return db.insert(resourceMembership).values({
          resourceType: "memory",
          resourceId: memoryId,
          allUserId: targetUser.id,
          permissions: permissions || "view",
          createdAt: new Date(),
        });
      })
    );

    return NextResponse.json({ success: true, memberships });
  }

  if (type === "public_link") {
    // Generate unique token
    const token = crypto.randomBytes(32).toString("hex");

    const publicLink = await db.insert(memoryPublicLinks).values({
      memoryId,
      token,
      createdBy: allUserRecord.id,
      expiresAt: expiresAt ? new Date(expiresAt) : null,
    });

    const shareUrl = `${process.env.NEXT_PUBLIC_APP_URL}/shared/${token}`;

    return NextResponse.json({
      success: true,
      link: shareUrl,
      token,
    });
  }
}
```

#### 2.2 Public Link Access

```typescript
// src/app/shared/[token]/page.tsx
export default async function SharedMemoryPage({ params }: { params: Promise<{ token: string }> }) {
  const { token } = await params;

  const publicLink = await db.query.memoryPublicLinks.findFirst({
    where: and(
      eq(memoryPublicLinks.token, token),
      eq(memoryPublicLinks.isActive, true),
      or(isNull(memoryPublicLinks.expiresAt), gt(memoryPublicLinks.expiresAt, new Date()))
    ),
    with: {
      memory: {
        with: {
          assets: true,
        },
      },
    },
  });

  if (!publicLink) {
    return <div>Link expired or not found</div>;
  }

  return (
    <div className="container mx-auto py-8">
      <h1 className="text-2xl font-bold mb-4">Shared Memory: {publicLink.memory.title}</h1>
      <MemoryViewer memory={publicLink.memory} />
    </div>
  );
}
```

### Phase 3: Integration & Testing

**Duration**: 2-3 hours

#### 3.1 Dashboard Integration

```typescript
// src/app/[lang]/dashboard/page.tsx
const [shareModalOpen, setShareModalOpen] = useState(false);
const [selectedMemory, setSelectedMemory] = useState<Memory | null>(null);

const handleShare = (memory: Memory) => {
  setSelectedMemory(memory);
  setShareModalOpen(true);
};

// In MemoryGrid
<ContentCard
  // ... other props
  onShare={handleShare}
/>;
```

#### 3.2 Share Management

```typescript
// src/components/memory/existing-shares-list.tsx
function ExistingSharesList({ memory }: { memory: Memory }) {
  const { data: shares, refetch } = useQuery({
    queryKey: ["memory-shares", memory.id],
    queryFn: () => fetchMemoryShares(memory.id),
  });

  const revokeShare = useMutation({
    mutationFn: (shareId: string) => revokeMemoryShare(memory.id, shareId),
    onSuccess: () => refetch(),
  });

  return (
    <div className="space-y-2">
      {shares?.map((share) => (
        <div key={share.id} className="flex items-center justify-between p-2 border rounded">
          <div>
            <span className="font-medium">{share.userEmail}</span>
            <span className="text-sm text-muted-foreground ml-2">{share.permissions}</span>
          </div>
          <Button size="sm" variant="destructive" onClick={() => revokeShare.mutate(share.id)}>
            Revoke
          </Button>
        </div>
      ))}
    </div>
  );
}
```

## Acceptance Criteria

### Functional Requirements

- [ ] Share modal opens from memory card share button
- [ ] Email sharing creates resourceMembership entries
- [ ] Public link generation creates secure tokens
- [ ] Public links work without authentication
- [ ] Existing shares can be viewed and revoked
- [ ] Share permissions (view/edit) are respected

### Technical Requirements

- [ ] API endpoints handle authentication and authorization
- [ ] Database schema supports public links and memberships
- [ ] Token generation is cryptographically secure
- [ ] Public links can be expired
- [ ] Share status updates in real-time

### UI/UX Requirements

- [ ] Modal is responsive and accessible
- [ ] Email input supports multiple recipients
- [ ] Public links can be easily copied
- [ ] Clear visual feedback for share actions
- [ ] Error states are handled gracefully

## Dependencies

### Database

- [ ] `memory_public_links` table creation
- [ ] Index on token field for performance
- [ ] Foreign key constraints to memories table

### API Layer

- [ ] Authentication middleware
- [ ] Email validation utilities
- [ ] Token generation utilities
- [ ] Public link access middleware

### Frontend

- [ ] Modal component library
- [ ] Form validation
- [ ] Clipboard API integration
- [ ] Toast notifications

## Security Considerations

### Access Control

- Only memory owners can create shares
- Public links require valid tokens
- Expired links are automatically invalidated
- Share permissions are enforced at API level

### Data Protection

- Tokens are cryptographically secure
- Email addresses are validated
- Public links can be revoked
- Audit trail for share actions

## Related Issues

- [Memory deletion dashboard not updating](./done/memory-deletion-dashboard-not-updating.md) - Similar modal integration patterns
- [Folder edit strategy](./open/folder-edit-strategy.md) - Modal component reuse
- [Access control architecture](./open/access-control-architecture-decision.md) - Permission system foundation

## Priority

**High** - Core sharing functionality for user collaboration

## Estimated Effort

- **Modal Component**: 3-4 hours
- **API Implementation**: 4-5 hours
- **Integration & Testing**: 2-3 hours
- **Total**: 9-12 hours

## Future Enhancements

### Phase 2 Features

- **Contact Management**: User contact list integration
- **Bulk Sharing**: Share multiple memories at once
- **Share Analytics**: Track link usage and access
- **Advanced Permissions**: Time-limited access, download restrictions

### Phase 3 Features

- **Share Notifications**: Email notifications for new shares
- **Share Comments**: Collaborative commenting on shared memories
- **Share Groups**: Predefined sharing groups
- **Share Templates**: Reusable sharing configurations

## Notes

- Public links should be long-lived but revocable
- Email sharing requires user account creation flow
- Share permissions should be granular (view, edit, delete)
- Consider rate limiting for public link generation
- Implement proper error handling for invalid tokens
