import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import App from '@/App'

import '@fortawesome/fontawesome-free/css/all.min.css'
import './index.scss'

const rootNode = document.getElementById('root')

if (rootNode) {
  createRoot(rootNode).render(
    <StrictMode>
      <App />
    </StrictMode>,
  )
}
