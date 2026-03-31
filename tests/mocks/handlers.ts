import { http, HttpResponse } from 'msw'
import { setupServer } from 'msw/node'

export const handlers = [
  http.get('https://api.example.com/users', () => {
    return HttpResponse.json([
      { id: 1, name: 'John Doe', email: 'john@example.com' },
      { id: 2, name: 'Jane Smith', email: 'jane@example.com' },
    ])
  }),

  http.get('https://api.example.com/users/:id', ({ params }) => {
    const { id } = params
    return HttpResponse.json({
      id: Number(id),
      name: 'John Doe',
      email: 'john@example.com',
    })
  }),

  http.post('https://api.example.com/users', async ({ request }) => {
    const body = await request.json()
    return HttpResponse.json({
      id: 3,
      ...body as object,
    }, { status: 201 })
  }),

  http.put('https://api.example.com/users/:id', async ({ request, params }) => {
    const body = await request.json()
    return HttpResponse.json({
      id: Number(params.id),
      ...body as object,
    })
  }),

  http.delete('https://api.example.com/users/:id', () => {
    return HttpResponse.json({ success: true })
  }),

  http.get('https://api.example.com/posts', () => {
    return HttpResponse.json([
      { id: 1, title: 'Post 1', body: 'Content 1' },
      { id: 2, title: 'Post 2', body: 'Content 2' },
    ])
  }),

  http.post('https://api.example.com/auth/login', async ({ request }) => {
    const body = await request.json() as { username: string; password: string }
    if (body.username === 'admin' && body.password === 'password') {
      return HttpResponse.json({
        token: 'mock-jwt-token-12345',
        user: { id: 1, username: 'admin' },
      })
    }
    return HttpResponse.json({ error: 'Invalid credentials' }, { status: 401 })
  }),

  http.get('https://api.example.com/error', () => {
    return HttpResponse.json({ error: 'Internal Server Error' }, { status: 500 })
  }),

  http.get('https://api.example.com/slow', async () => {
    await new Promise(resolve => setTimeout(resolve, 2000))
    return HttpResponse.json({ message: 'Slow response' })
  }),
]

export const server = setupServer(...handlers)
