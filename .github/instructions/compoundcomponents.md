---
applyTo: "**/*.{tsx,jsx}, **/components/**/*.{ts,tsx,js,jsx}"
---

This guide explores the **Compound Component Pattern** using React and TypeScript. This senior-level architecture allows you to build highly flexible, declarative components by leveraging **Inversion of Control** and the **Context API**.

## Why Compound Components?

In traditional "prop-heavy" components, you often end up passing numerous flags (e.g., `showIcon`, `isClosable`, `footerText`) to a single parent component. This makes the component rigid and hard to maintain.

Compound components solve this by breaking the UI into smaller sub-components that share state implicitly, allowing the user of your component to decide the layout.

## Core Implementation Pattern

A robust TypeScript implementation involves four main parts: the Context, the Main Parent, the Sub-components, and the Static Property Assignment.

### 1. Define Types and Context
First, define the shape of your shared state and create a context. Using a custom hook to consume this context ensures type safety and prevents runtime errors.

```typescript
import React, { createContext, useContext, useState, ReactNode } from 'react';

// 1. Define the state shape
interface TabsContextType {
  activeTab: string;
  setActiveTab: (id: string) => void;
}

// 2. Create context with a null default
const TabsContext = createContext<TabsContextType | null>(null);

// 3. Custom hook for safe consumption
function useTabsContext() {
  const context = useContext(TabsContext);
  if (!context) {
    throw new Error('Tabs sub-components must be rendered within a <Tabs /> provider');
  }
  return context;
}
```

### 2. The Parent Component
The parent manages the logic and provides it to the children through the provider.

```typescript
interface TabsProps {
  children: ReactNode;
  defaultTab: string;
}

const TabsRoot = ({ children, defaultTab }: TabsProps) => {
  const [activeTab, setActiveTab] = useState(defaultTab);

  return (
    <TabsContext.Provider value={{ activeTab, setActiveTab }}>
      <div className="tabs-container">{children}</div>
    </TabsContext.Provider>
  );
};
```

### 3. Sub-components
Each sub-component uses the `useTabsContext` hook to interact with the shared state.

```typescript
interface TabProps {
  id: string;
  children: ReactNode;
}

const TabTrigger = ({ id, children }: TabProps) => {
  const { activeTab, setActiveTab } = useTabsContext();
  return (
    <button 
      onClick={() => setActiveTab(id)}
      className={activeTab === id ? 'active' : ''}
    >
      {children}
    </button>
  );
};

const TabContent = ({ id, children }: TabProps) => {
  const { activeTab } = useTabsContext();
  return activeTab === id ? <div className="tab-pane">{children}</div> : null;
};
```

### 4. Component Composition (The TypeScript Way)
To enable the clean `Tabs.Trigger` dot-notation and ensure full type safety, assign the sub-components as static properties.

```typescript
// Define an interface for the final composed component
interface TabsComponent extends React.FC<TabsProps> {
  Trigger: typeof TabTrigger;
  Content: typeof TabContent;
}

// Cast the Root component to the final interface
export const Tabs = TabsRoot as TabsComponent;

Tabs.Trigger = TabTrigger;
Tabs.Content = TabContent;
```

## Practical Comparison

The following table demonstrates how this pattern simplifies the developer experience for the consumer.

| Feature | Traditional Approach (Prop-heavy) | Compound Pattern (Composition) |
| :--- | :--- | :--- |
| **API** | `<Tabs data={items} variant="dark" />` | `<Tabs><Tabs.Trigger id="1">...</Tabs></Tabs>` |
| **Customization** | Requires new props for every UI tweak . | Naturally flexible; reorder children easily . |
| **Logic** | Logic and UI are tightly coupled . | UI structure is decoupled from logic . |
| **Usage** | Hard to add custom components between tabs . | User can insert any JSX between sub-components . |

## Advanced: Logic with `React.Children`
Sometimes, you want the parent to control which children are rendered based on conditions (like an onboarding flow). Use `React.Children.toArray` to filter children dynamically.

```typescript
export const Onboarding = ({ children }: { children: ReactNode }) => {
  const [currentStep, setCurrentStep] = useState(0);
  
  // Convert children to array and filter out conditionally hidden steps
  const steps = React.Children.toArray(children).filter(
    (child: any) => child.props.when !== false
  );

  return (
    <div className="onboarding-stepper">
      {steps[currentStep]}
    </div>
  );
};
```

## Best Practices
- **Custom Hooks**: Always use a custom hook to consume context and throw a clear error if used outside the provider.
- **Dot Notation**: Use static properties (e.g., `Accordion.Item`) to keep the related components logically grouped in the global namespace.
- **Don't Over-engineer**: Only use this pattern for components that actually need flexibility. For simple inputs or static labels, standard props are more efficient.