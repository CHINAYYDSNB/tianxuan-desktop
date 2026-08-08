import { useParams } from "react-router-dom";
import { useEffect } from "react";

import { useHostStore } from "../../stores/hostStore";
import HostForm from "./HostForm";

export default function HostEditPage() {
  const { id } = useParams();
  const { hosts, load } = useHostStore();

  useEffect(() => {
    if (!hosts.length) load();
  }, [hosts.length, load]);

  const host = hosts.find((h) => h.id === id);
  if (!host) {
    return (
      <div className="p-6 text-sm text-zinc-500">主机不存在或未加载</div>
    );
  }
  return <HostForm editing={host} />;
}
