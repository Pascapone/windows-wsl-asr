import type { AudioDeviceInfo } from '../lib/types'

type Props = {
  devices: AudioDeviceInfo[]
  selectedId: string | null
  onSelect: (deviceId: string | null) => void
  onRefresh: () => void
}

export function DeviceSelect({ devices, selectedId, onSelect, onRefresh }: Props) {
  return (
    <div className="stack">
      <label>
        <span>Input device</span>
        <select value={selectedId ?? ''} onChange={(event) => onSelect(event.target.value || null)}>
          <option value="">Default device</option>
          {devices.map((device) => (
            <option key={device.id} value={device.id}>
              {device.name}{device.isDefault ? ' (default)' : ''}
            </option>
          ))}
        </select>
      </label>
      <button className="secondary" onClick={onRefresh}>
        Refresh devices
      </button>
    </div>
  )
}
