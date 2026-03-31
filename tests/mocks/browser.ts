import { http, HttpResponse } from 'msw'
import { setupWorker } from 'msw/browser'

export const browserHandlers = [
  http.get('https://api.example.com/users', () => {
    return HttpResponse.json([
      { id: 1, name: 'John Doe', email: 'john@example.com' },
      { id: 2, name: 'Jane Smith', email: 'jane@example.com' },
    ])
  }),

  http.post('https://api.example.com/users', async ({ request }) => {
    const body = await request.json()
    return HttpResponse.json({ id: 3, ...body as object }, { status: 201 })
  }),
]

export function startWorker() {
  const worker = setupWorker(...browserHandlers)
  worker.start({
    onUnhandledRequest: 'bypass',
    serviceWorker: {
      url: '/mockServiceWorker.js',
    },
  })
  return worker
}
