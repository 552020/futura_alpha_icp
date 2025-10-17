# Mobile Overflow Debugging Guide

## 🎯 **Quick Summary**

One (or more) children is wider than the visual viewport, so the page gets a horizontal layout viewport bigger than the header.

**Fix = enforce the correct viewport and stop any min-width/100vw/w-max offenders.**

## 🔍 **Quick Checks (Do These in Order)**

### **1. Viewport Tag**

In `<head>` ensure exactly:

```html
<meta name="viewport" content="width=device-width, initial-scale=1" />
```

If it's missing/wrong, you'll see classic "container wider than header" behavior.

### **2. Global Overflow Guard (Temporary, Helps Debugging)**

```html
<html class="overflow-x-hidden">
  <body class="overflow-x-hidden"></body>
</html>
```

If this "fixes" it, you do have an overflowing child to hunt down.

### **3. 100vw Usage**

Search for `w-screen`, `w-[100vw]`, `max-w-[100vw]`, plain CSS `width: 100vw`. Replace with `w-full` on normal layout containers. `100vw` includes scrollbar and often causes a few extra pixels of overflow on mobile.

### **4. min-width / w-max / whitespace-nowrap**

On the toolbar row (Add Folder / Add File / Clear All / view toggle), make sure none of the wrappers has:

- `min-w-[...px]`
- `w-max`
- `whitespace-nowrap` on long, unbreakable labels

If present, replace with `min-w-0 w-full` on the row container and `shrink` on the buttons:

```tsx
<div className="flex min-w-0 flex-wrap gap-2">
  <button className="shrink-0">Add Folder</button>
  ...
</div>
```

### **5. Tailwind `.container`**

Tailwind's `container` is `width: 100%` on mobile; it only gains `max-width` at breakpoints. If yours shows ~544px on a 375px viewport, something upstream is creating a larger containing block. Add:

```tsx
<div className="container mx-auto px-6 max-w-full">
```

Or skip `container` and do:

```tsx
<div className="mx-auto w-full max-w-screen-lg px-6 sm:px-8">
```

### **6. Sticky Header Sizing**

Make sure the header's immediate parent isn't narrower than the page. Use:

```tsx
<header className="sticky top-0 z-50 w-full">
  <div className="mx-auto w-full max-w-full px-4">...</div>
</header>
```

Avoid `w-screen` in the header.

## 🔧 **How to Find the Exact Offender (Copy-Paste in Console)**

### **List Elements That Extend Past the Viewport:**

```javascript
[...document.querySelectorAll("*")]
  .filter((el) => el.getBoundingClientRect().right > document.documentElement.clientWidth)
  .map((el) => el);
```

### **See Current Viewport Meta:**

```javascript
document.querySelector("meta[name=viewport]")?.content;
```

### **Quick Visual Tracer:**

```javascript
[...document.querySelectorAll("*")].forEach((el) => {
  const r = el.getBoundingClientRect();
  if (r.right > document.documentElement.clientWidth) {
    el.style.outline = "2px solid red";
  }
});
```

## 🎯 **Likely Culprits in Your Screenshots**

- **The filters/toolbar row** (buttons + view toggle) looks like an inline/flex row that doesn't wrap (`flex-nowrap` or `whitespace-nowrap`) and forces the parent wider. Ensure `flex-wrap` and `min-w-0` on the row container; `shrink-0` only on items that must not shrink.
- **Any element using `w-screen` or `w-max`** inside the main container.

## 🛠️ **Minimal Tailwind Fixes to Try**

### **1. On the Page Wrapper:**

```tsx
<div className="w-full max-w-full overflow-x-hidden">
```

### **2. On the Main Container:**

```tsx
<div className="mx-auto w-full max-w-full px-6 sm:px-8">
```

### **3. On the Toolbar Row:**

```tsx
<div className="flex min-w-0 flex-wrap items-center gap-2">
  {/* each button */}
  <button className="shrink-0">Add Folder</button>
</div>
```

## 📋 **Information to Provide for Specific Help**

If you want specific help, pass:

1. **The `<meta name="viewport">` line**
2. **The header component JSX** (just the outer wrappers)
3. **The container + toolbar JSX**

And we can point to the exact line to change.

## 🎯 **Expected Results**

After applying these fixes:

- ✅ No horizontal scrolling on mobile
- ✅ Header and content widths align perfectly
- ✅ All elements respect the viewport width
- ✅ Touch interactions work smoothly
- ✅ Layout is consistent across all screen sizes

## 🔄 **Prevention Checklist**

- [ ] Always use `w-full` instead of `w-screen`
- [ ] Add `flex-wrap` to flex containers
- [ ] Use `min-w-0` on flex items that need to shrink
- [ ] Avoid `whitespace-nowrap` on long text
- [ ] Test on actual mobile devices regularly
- [ ] Use the console debugging commands to verify fixes
