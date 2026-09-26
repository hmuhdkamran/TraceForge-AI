import { statusLabel, statusColor } from '../../utils/status';

interface Props {
  status: string;
  large?: boolean;
}

export function StatusBadge({ status, large }: Props) {
  return (
    <span
      className={`inline-flex items-center border rounded px-2 py-0.5 font-medium ${
        large ? 'text-sm' : 'text-xs'
      } ${statusColor(status)}`}
    >
      {statusLabel(status)}
    </span>
  );
}
