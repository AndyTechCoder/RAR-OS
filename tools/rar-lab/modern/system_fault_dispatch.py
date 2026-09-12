"""Fixed trusted-main dispatcher; mode comes only from the literal job matrix."""
import importlib.util,os
from pathlib import Path
def main():
    mode=os.environ.get("RAR_SYSTEM_TRANSACTION")
    if mode not in ("install","repair"):raise ValueError("fixed System transaction")
    path=Path(__file__).resolve().with_name("runtime_controller.py")
    if path.is_symlink() or not path.is_file():raise ValueError("fixed controller")
    spec=importlib.util.spec_from_file_location("runtime_controller",path)
    controller=importlib.util.module_from_spec(spec);spec.loader.exec_module(controller)
    controller.main("system-"+mode+"-faults")
if __name__=="__main__":main()
