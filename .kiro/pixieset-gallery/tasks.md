# Implementation Plan

## MVP Core Features (Phase 1)

- [x] 1. Set up mock data system for gallery development (PARALLEL TASK - can be done independently)

  - Create mock gallery data generator script with different gallery sizes and image formats
  - Implement environment variable toggle for mock vs real data
  - Generate galleries with 3, 10, 25, 50+ images in landscape, portrait, and mixed formats
  - Create sample gallery images in public/mock/gallery directory structure
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6_

- [x] 2. Create core gallery TypeScript interfaces and types

  - Define Gallery, GalleryImage, and FolderInfo interfaces
  - Create gallery service utility functions (mock data flag controlled by main gallery component)
  - Set up error handling types for gallery operations
  - Add track(event, props) analytics shim for future implementation
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [x] 3. Implement GalleryPage component

  - Create main gallery overview page at /[lang]/gallery route
  - Display user's own galleries in responsive grid layout
  - Show gallery thumbnails, titles, and creation dates
  - Implement loading states and error handling
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 4. Build GalleryCard component for gallery list display

  - Create reusable gallery card using shadcn/ui Card component (integrated into GalleryPage)
  - Use shadcn/ui Badge for gallery stats (image count, privacy status)
  - Implement hover effects with shadcn/ui styling patterns
  - Handle missing cover images with placeholder logic (first positioned → latest → placeholder)
  - Add gallery metadata display with consistent typography
  - _Requirements: 15.1, 15.2, 15.3_

- [x] 5. Create GalleryViewPage component for individual gallery viewing

  - Use same mock data pattern as dashboard (const USE_MOCK_DATA = true at component level)
  - Implement same top-bar component pattern as dashboard for consistent UI
  - Use current grid layout (refinements can be added as later tasks)
  - Add basic edit/delete buttons for gallery owners
  - Handle public gallery access via /g/[id] route (no auth required)
  - Handle private gallery access via /[lang]/gallery/[id] route (auth required)
  - Remove lightbox functionality (will be separate task 6)
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 14.1, 14.3, 14.5, 15.4, 15.5_

- [ ] 6. Implement ImageLightbox component for full-screen viewing

  - Investigate reusing existing memory viewer component from dashboard
  - Create modal lightbox using shadcn/ui Dialog component with focus trap
  - Add previous/next navigation with arrow keys and prefetch adjacent images
  - Implement mandatory focus trap and aria-hidden on background
  - Add aria-live announcements for screen reader navigation
  - Include image title and description display
  - Prefetch next/prev image URLs on lightbox open for smooth navigation
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 13.1, 13.2, 13.3, 13.4_

- [ ] 7. Build CreateGalleryModal component

  - Create modal dialog using shadcn/ui Dialog component
  - Use shadcn/ui Form components for gallery title and description input
  - Fetch and display available folders containing images using shadcn/ui Card components
  - Show image count preview for each folder with shadcn/ui Badge
  - Use shadcn/ui Button components for actions
  - Return created gallery ID synchronously and redirect to /[lang]/gallery/[id]
  - Handle zero-image folders with friendly toast and keep user in place
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 11.1, 11.2, 11.3, 11.4, 11.5, 11.6_

- [ ] 8. Integrate "Create Gallery" button in dashboard

  - Add "Create Gallery" button to dashboard top bar or main area
  - Connect button to CreateGalleryModal component
  - Handle folder fetching and gallery creation flow
  - Show appropriate messaging when no folders exist
  - _Requirements: 2.1, 2.5, 11.5_

- [ ] 9. Implement basic gallery privacy controls

  - Add privacy toggle (private/public) in gallery settings
  - Update gallery privacy via existing PATCH /api/galleries/[id] endpoint
  - Show privacy status in gallery list and view
  - Handle public gallery access via /g/[id] URLs
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 15.4, 15.5_

- [ ] 10. Update sidebar navigation for galleries
  - Modify sidebar to highlight "Gallery" when on gallery pages
  - Ensure consistent navigation behavior across gallery routes
  - Test navigation flow between dashboard, gallery list, and individual galleries
  - Lazy-load gallery list after mount (debounced) to avoid blocking dashboard first paint
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

## Enhanced Features (Phase 2)

- [ ] 11. Implement gallery editing functionality

  - Create GalleryEditModal using shadcn/ui Dialog and Form components
  - Use shadcn/ui Input and Textarea for title and description editing
  - Add ability to add/remove images from gallery with shadcn/ui Select components
  - Implement gallery deletion with shadcn/ui AlertDialog confirmation
  - Handle empty gallery reuse logic for duplicate titles
  - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5, 11.7_

- [ ] 12. Add infinite scroll for large galleries

  - Implement infinite scroll pattern from dashboard for gallery view
  - Add pagination to gallery image loading
  - Show loading states during scroll-triggered loads
  - Handle scroll position restoration
  - _Requirements: 14.2, 14.3_

- [ ] 13. Implement image preloading and performance optimization

  - Preload first row of images in gallery view
  - Prefetch adjacent images in lightbox navigation
  - Add skeleton loading states for images
  - MANDATORY: Store image dimensions in images.metadata on upload or first view
  - Ensure gallery grid always renders with known aspect ratios to prevent CLS
  - _Requirements: 14.4, 14.5_

- [ ] 14. Create enhanced sidebar with multiple galleries

  - Implement collapsible "Gallery" section in sidebar
  - Show scrollable list of user's galleries
  - Add thumbnail previews on hover
  - Create separate "Shared Galleries" subsection
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6_

- [ ] 15. Add download controls and sharing features
  - Implement "Allow downloads" toggle in gallery settings
  - Show/hide download buttons based on gallery settings
  - Add basic gallery sharing with individual users
  - Generate shareable public gallery links
  - _Requirements: 6.5, 13.5, 13.6, 13.7_

## Advanced Features (Phase 3)

- [ ] 16. Implement comprehensive analytics tracking

  - Track gallery creation, privacy changes, and sharing events
  - Monitor image viewing and navigation in lightbox
  - Measure performance metrics (load times, scroll triggers)
  - Track user journey from creation to first publish
  - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5_

- [ ] 17. Add mobile-optimized experience

  - Implement touch gestures for lightbox navigation
  - Optimize gallery grid for mobile viewports
  - Add mobile-friendly edit interface
  - Test and optimize touch target sizes
  - _Requirements: Mobile experience considerations from feedback_

- [ ] 18. Create folder management API endpoints and enhance existing gallery APIs

  - Implement GET /api/folders/with-images endpoint
  - Add pagination support for gallery images API
  - Enhance GET /api/galleries/[id] to return only published items for public routes
  - Add PUT /api/galleries/[id]/items/reorder endpoint for bulk position updates
  - Ensure unique position per (galleryId, position) and handle gapless reordering
  - Handle deleted images referenced in gallery_items (cascade cleanup or soft-delete)
  - _Requirements: 2.2, 2.3, 11.2, 11.3_

- [ ] 19. Implement SEO-friendly URLs and public gallery features

  - Add support for /u/[slug]/gallery/[slugOrId] URL pattern
  - Implement proper meta tags for public galleries
  - Add Open Graph support for gallery sharing
  - Create gallery sitemap generation
  - _Requirements: 15.6, Future SEO considerations_

- [ ] 20. Add comprehensive error handling and edge cases
  - Implement gallery error boundaries with recovery options
  - Handle network failures gracefully with retry mechanisms
  - Add offline state detection and messaging
  - Create fallback UI for broken images or missing galleries
  - _Requirements: Error handling from design document_

## Testing and Quality Assurance

- [ ] 21. Write unit tests for gallery components

  - Test GalleryCard rendering with different data scenarios
  - Test ImageLightbox keyboard navigation and accessibility
  - Test CreateGalleryModal form validation and submission
  - Test gallery privacy controls and state management
  - _Requirements: Testing strategy from design document_

- [ ] 22. Implement integration tests for gallery workflows

  - Test complete gallery creation flow from dashboard
  - Test gallery viewing and navigation between images
  - Test privacy setting changes and access control
  - Test error scenarios and recovery flows
  - Test critical edge cases: empty galleries, mixed EXIF orientations, deleted images in public galleries
  - Test large gallery performance (100+ images) with pagination and lightbox indexing
  - _Requirements: Testing strategy from design document_

- [ ] 23. Perform accessibility and performance testing
  - Verify screen reader compatibility and keyboard navigation
  - Test focus management in modals and lightbox
  - Measure and optimize large gallery rendering performance
  - Validate color contrast and touch target accessibility
  - _Requirements: Accessibility testing from design document_

## Notes

- **Mock Data Toggle**: Use flag in main gallery component (similar to dashboard pattern)
- **API Integration**: Leverage existing `/api/galleries` endpoints with enhancements for public access control
- **Database Schema**: Defer schema enhancements (coverImageId, allowDownloads) to Phase 2
- **Cover Image Logic**: Compute cover image on read using position → createdAt → placeholder logic
- **Error Boundaries**: Implement at gallery page level with graceful fallback UI
- **Performance**: Focus on Next.js Image optimization and lazy loading for MVP

## Critical Technical Requirements

- **Route Structure**: Public galleries on `/g/:id` (no auth), private on `/[lang]/gallery/[id]` (auth required)
- **Access Control**: API must filter published items only for public routes, prevent auth leakage
- **Image Dimensions**: MANDATORY storage of width/height in metadata to prevent layout shift
- **Position Integrity**: Unique positions per gallery, gapless after deletes, bulk reorder support
- **Focus Management**: Mandatory focus trap in lightbox with aria-hidden background
- **Prefetching**: Prefetch adjacent images on lightbox open for smooth navigation
- **Analytics Shim**: Implement track(event, props) function now for future analytics integration

## Component Reuse Strategy

- **shadcn/ui Components**: Use existing shadcn/ui components (Dialog, Card, Button, Form, Badge, etc.) for consistent UI
- **Memory Viewer**: Investigate reusing `src/components/memory-viewer.tsx` for ImageLightbox to avoid duplication
- **MemoryGrid Pattern**: Consider adapting existing `MemoryGrid` component patterns for gallery image display
- **Form Components**: Reuse existing form patterns from dashboard and onboarding components

## Parallel Development Tasks

Tasks marked as "PARALLEL TASK" can be worked on simultaneously by different team members:

- **Task 1**: Mock data generation (independent script creation)
- **Task 2**: TypeScript interfaces (foundational types)
- **Task 18**: API endpoints (backend-focused work)

Other tasks have dependencies and should be completed sequentially within each phase.
