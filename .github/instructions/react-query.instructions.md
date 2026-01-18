---
applyTo: "**/queries/**/*.{ts,tsx,js,jsx}, **/mutations/**/*.{ts,tsx,js,jsx}, **/api/**/*.{ts,tsx,js,jsx}, **/hooks/**/use*.{ts,tsx,js,jsx}, **/*query.{ts,tsx,js,jsx}, **/*Query.{ts,tsx,js,jsx}, **/*mutation.{ts,tsx,js,jsx}, **/*Mutation.{ts,tsx,js,jsx}, **/services/**/*.{ts,tsx,js,jsx}"
---

## TanStack Query (React Query) v5 Best Practices and Standards

Scope: This rule set guides AI coding assistance for React apps using TanStack Query v5. It encodes patterns, constraints, and examples drawn from the project’s `FullGuide/`.

### Goals

- Maximize correctness and UX for server-state.
- Keep components render-efficient and predictable.
- Make cache keys stable and domain-driven.
- Prefer declarative orchestration over imperative refetching.

### Core Standards

1. Client vs Server State

- Treat server data as cached, not owned by the UI.
- Do NOT copy query results to local component state unless intentionally freezing (e.g., form defaults).

```tsx
// Anti-pattern: copies server state into local state
const { data } = useQuery({
  queryKey: ["user", id],
  queryFn: () => api.user(id),
});
const [user, setUser] = useState(data);

// Preferred: read from cache directly
const { data: user } = useQuery({
  queryKey: ["user", id],
  queryFn: () => api.user(id),
});
```

2. Query Keys

- Use array keys with stable primitives; include every input used by `queryFn`.
- Keep key factories in one place.

```ts
export const keys = {
  users: () => ["users"] as const,
  user: (id: string) => ["user", id] as const,
};

useQuery({ queryKey: keys.user(id), queryFn: () => api.user(id) });
```

3. Query Function Context

- Prefer extracting params from `queryKey` in `queryFn` for single source of truth.

```tsx
async function query({ queryKey }: { queryKey: [string, string] }) {
  const [, id] = queryKey;
  return api.user(id);
}
```

4. Defaults and Options

- Tune `staleTime` primarily; rarely adjust `gcTime`.
- Keep `refetchOnWindowFocus: true` for production; disable case-by-case only.
- Use `enabled` to gate dependent queries.

```tsx
const q = useQuery({
  queryKey: ["projects", orgId],
  queryFn: () => api.projects(orgId!),
  enabled: !!orgId,
  staleTime: 30_000,
});
```

5. Transformations and Selectors

- Prefer `select` to shape data near the cache; keep transforms pure and structurally shared.

```tsx
useQuery({
  queryKey: ["repos"],
  queryFn: api.repos,
  select: (repos) => {
    const next = [...repos].sort((a, b) => b.stars - a.stars).slice(0, 5);
    return next;
  },
});
```

6. Status and Error Handling

- Use `status` for data state and `fetchStatus` for transport state.
- Configure retries via function to avoid retrying 4xx.

```tsx
useQuery({
  queryKey: ["user", id],
  queryFn: api.user,
  retry: (count, err: any) => (err?.status >= 500 ? count < 2 : false),
});
```

7. Mutations and Invalidation

- Use `invalidateQueries` for correctness after writes; use `setQueryData` for low-latency UX when safe.

```tsx
const qc = useQueryClient();
const createTodo = useMutation({
  mutationFn: api.createTodo,
  onSuccess: (newTodo) => {
    qc.setQueryData<Todo[]>(["todos"], (prev) =>
      prev ? [newTodo, ...prev] : [newTodo]
    );
    // Consider invalidation if other lists/pages may be affected
    // qc.invalidateQueries({ queryKey: ['todos'] })
  },
});
```

8. Optimistic Updates with Rollback

- Cancel queries, snapshot previous, apply optimistic change, rollback on error, then settle with invalidation.

```tsx
const toggle = useMutation({
  mutationFn: ({ id, done }: { id: number; done: boolean }) =>
    api.updateTodo({ id, done }),
  onMutate: async (patch) => {
    await qc.cancelQueries({ queryKey: ["todos"] });
    const previous = qc.getQueryData<Todo[]>(["todos"]);
    qc.setQueryData<Todo[]>(["todos"], (old) =>
      (old ?? []).map((t) =>
        t.id === patch.id ? { ...t, done: patch.done } : t
      )
    );
    return { previous };
  },
  onError: (_e, _v, ctx) => {
    if (ctx?.previous) qc.setQueryData(["todos"], ctx.previous);
  },
  onSettled: () => {
    qc.invalidateQueries({ queryKey: ["todos"] });
  },
});
```

9. Placeholder vs Initial Data

- `placeholderData` improves perceived latency without seeding cache.
- `initialData` seeds cache and is initially fresh.

```tsx
useQuery({
  queryKey: ["repo", id],
  queryFn: () => api.repo(id),
  placeholderData: cachedShallow,
});
useQuery({
  queryKey: ["repo", id],
  queryFn: () => api.repo(id),
  initialData: () => bootstrapRepo(id),
});
```

10. Infinite Queries and WebSockets

- Use `useInfiniteQuery` with `getNextPageParam`.
- For socket events, update the right page with `setQueryData` and idempotent merges.

```tsx
queryClient.setQueryData<{ pages: Page[]; pageParams: unknown[] }>(
  ["items"],
  (data) =>
    !data
      ? data
      : {
          ...data,
          pages: data.pages.map((p) => ({
            ...p,
            items: p.items.map((i) => (i.id === ev.id ? { ...i, ...ev } : i)),
          })),
        }
);
```

11. Offline and Forms

- Use `networkMode: 'offlineFirst'` for queuing.
- For forms, freeze with `staleTime: Infinity` and seed updates on submit via `setQueryData`.

### Performance Guidance

- Derive in `select` to reduce per-render work.
- Use narrow keys and narrow invalidations.
- Avoid passing large server data via React Context; prefer direct subscriptions via hooks.

### TypeScript Guidance

- Type `queryFn` return values; let `select` inference flow.
- Provide typed key factories with `as const`.

### Do/Don’t Checklist

- Do: co-locate custom hooks with fetchers and options.
- Do: treat query keys like dependency arrays.
- Do: prefer `enabled` for dependencies over imperative `refetch`.
- Don’t: mirror server data into global client stores or component state.
- Don’t: use `setQueryData` as a local store; background refetch can overwrite.
