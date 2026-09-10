"""Fixed trusted-main mounted-error diagnostic; no options."""
import importlib.util
from pathlib import Path
path=Path(__file__).with_name("runtime_controller.py")
if path.is_symlink() or not path.is_file():raise ValueError("fixed controller sibling")
spec=importlib.util.spec_from_file_location("runtime_controller",path)
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
if __name__=="__main__":module.main("mounted-error")
