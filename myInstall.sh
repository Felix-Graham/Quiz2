# to be run as root  
echo "Git add"
git add .
echo "Moving Vocab..."
cp vocab/* /home/rhemus/.local/share/frenchquiz/vocab/
echo "Removing Nix Profile"
nix profile remove quiz2
echo "Installing Nix Profile"
nix profile add
echo "Done"
