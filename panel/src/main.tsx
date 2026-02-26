import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import { ConfigProvider, theme } from 'antd'
import 'antd/dist/reset.css'

import App from '@/App'

import './index.scss'

const rootNode = document.getElementById('root')

if (rootNode) {
  createRoot(rootNode).render(
    <StrictMode>
      <ConfigProvider
        button={{
          autoInsertSpace: false,
        }}
        theme={{
          algorithm: theme.defaultAlgorithm,
          token: {
            colorPrimary: '#2f9a71',
            colorInfo: '#2f9a71',
            borderRadius: 12,
            borderRadiusLG: 16,
            borderRadiusSM: 10,
            fontFamily: 'Segoe UI Variable Text, PingFang SC, Microsoft YaHei, sans-serif',
          },
        }}
      >
        <App />
      </ConfigProvider>
    </StrictMode>,
  )
}
