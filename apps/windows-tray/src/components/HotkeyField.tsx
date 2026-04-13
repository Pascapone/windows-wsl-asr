import { useState } from 'react'

type Props = {
  value: string
  onChange: (value: string) => void
}

const modifierKeys = new Set(['Control', 'Shift', 'Alt', 'Meta'])

function normalizeKey(key: string) {
  if (key === ' ') return 'Space'
  if (key.length === 1) return key.toUpperCase()
  return key[0].toUpperCase() + key.slice(1)
}

export function HotkeyField({ value, onChange }: Props) {
  const [capturing, setCapturing] = useState(false)

  return (
    <div className="stack">
      <label>
        <span>Global hotkey</span>
        <input
          value={capturing ? 'Press key combination...' : value}
          readOnly
          onFocus={() => setCapturing(true)}
          onBlur={() => setCapturing(false)}
          onKeyDown={(event) => {
            if (!capturing) return
            event.preventDefault()

            const parts: string[] = []
            if (event.ctrlKey) parts.push('Ctrl')
            if (event.shiftKey) parts.push('Shift')
            if (event.altKey) parts.push('Alt')
            if (event.metaKey) parts.push('Meta')
            if (!modifierKeys.has(event.key)) parts.push(normalizeKey(event.key))

            if (parts.length > 1) {
              onChange(parts.join('+'))
              setCapturing(false)
            }
          }}
        />
      </label>
      <p className="hint">Feld fokussieren und neue Kombination drücken. Mindestens ein Modifier plus Taste.</p>
    </div>
  )
}
