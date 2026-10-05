import { Navigate, Route, Routes } from 'react-router'
import AppDetail from '@/pages/AppDetail'
import Discover from '@/pages/Discover'

function App() {
  return (
    <Routes>
      <Route path="/discover" element={<Discover />} />
      <Route path="/apps/:id" element={<AppDetail />} />
      <Route path="/" element={<Navigate to="/discover" replace />} />
      <Route path="*" element={<Navigate to="/discover" replace />} />
    </Routes>
  )
}

export default App