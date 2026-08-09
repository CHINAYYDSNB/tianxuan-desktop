import { useParams } from "react-router-dom";
import { useEffect } from "react";

import { usePanelStore } from "../../stores/panelStore";
import PanelForm from "./PanelForm";

export default function PanelEditPage() {
  const { id } = useParams();
  const { panels, load } = usePanelStore();

  useEffect(() => {
    if (!panels.length) load();
  }, [panels.length, load]);

  const panel = panels.find((p) => p.id === id);
  if (!panel) {
    return (
      <div className="p-6 text-sm text-zinc-500">面板不存在或未加载</div>
    );
  }
  return <PanelForm editing={panel} />;
}
