pub const SESSION: &str = r##"
    <script>
      $(document).ready(function () {
        $('#start-session').click(function () {
          $.ajax({
              url: '/session',
              type: 'PUT',
              data: JSON.stringify({
                "players": $('#player-select').val().map(x => Number(x)),
                "game_info": {
                  "games": Number($("#num-games-select").val()),
                  "players_per_team": Number($("#players-per-select").val()),
                  "stability": Number($("#stability-select").val())
                }
              }),
          }).done(function() {
            location.reload();
          });
        }); 
        $('#add-match').click(function () {
          $.ajax({
              url: '/add-match',
              type: 'PUT',
              data: JSON.stringify({
                "id": 0,
                "team1": $('#winners-select').val().map(x => Number(x)),
                "team2": $('#losers-select').val().map(x => Number(x)),
              }),
            }).done(function () {
                const successAddMatchToast = document.getElementById('add-match-toast');
                const toast = bootstrap.Toast.getOrCreateInstance(successAddMatchToast);
                toast.show();
            });
          });
        $('#generate-matches').click(function () {
          $.ajax({
              url: '/generate-matches',
              type: 'PUT',
              data: JSON.stringify($('#matchmake-select').val().map(x => Number(x))),
              success: function (data) {
                document.getElementById("match-area").innerHTML = data;
              },
          });
        });
        $('#add-session').click(function () {
          $.ajax({
              url: '/session',
              type: 'PATCH',
              data: JSON.stringify($('#session-select').val().map(x => Number(x))),
          }).done(function() {
            location.reload();
          });
        });
        $('#stop-session').click(function () {
          $.ajax({
              url: '/session',
              type: 'DELETE',
          }).done(function() {
            location.reload();
          });
        });
      });
    </script>
  </body>
</html>
    "##;

pub const PLAYER: &str = r##"
<script>
      $(document).ready(function () {
        $('#add-player').click(function () {
          const score = Number($("#player-score").val());
          $.ajax({
              url: '/players',
              type: 'POST',
              data: JSON.stringify({
                "name": $("#player-name").val(),
                "score": isNaN(score) ? null : score,
              }),
          }).done(function () {
            const successAddPlayerToast = document.getElementById('add-player-toast');
            const toast = bootstrap.Toast.getOrCreateInstance(successAddPlayerToast);
            toast.show();
          });
        });
      });
    </script>
  </body>
</html>
"##;
